use borsh::{BorshDeserialize, BorshSerialize};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use spl_associated_token_account::instruction as ata_instruction;
use spl_token::{instruction as token_instruction, state::Account as TokenAccount};

use crate::{
    errors::{AstrapeError, AstrapeResult},
    instructions::AstrapeInstruction,
    state::{AstrapeConfig, UserDeposit, UserDepositState},
};

// PDA seeds
pub const CONFIG_SEED: &[u8] = b"pool_config";
pub const AUTHORITY_SEED: &[u8] = b"authority";
pub const WITHDRAWAL_POOL_SEED: &[u8] = b"withdrawal_pool";

pub const MS_PER_SLOT: u64 = 440;
pub const SLOTS_PER_SEC: f64 = 1000.0 / MS_PER_SLOT as f64;
pub const SLOTS_PER_MONTH: f64 = SLOTS_PER_SEC * 30.0 * 24.0 * 60.0 * 60.0;
pub const SLOTS_PER_YEAR: f64 = SLOTS_PER_SEC * 365.0 * 24.0 * 60.0 * 60.0;

const PYTH_PRICE_UPDATE_DISCRIMINATOR: &'static [u8] = &[34, 241, 35, 99, 157, 126, 244, 205];

#[cfg(feature = "testnet")]
pub mod config_feature {
    pub mod admin {
        solana_program::declare_id!("EjYMbwtvCjAdMB2RPu45QKPBEE5gTPSJBktzTro5VigV");
    }
}
#[cfg(feature = "devnet")]
pub mod config_feature {
    pub mod admin {
        solana_program::declare_id!("EjYMbwtvCjAdMB2RPu45QKPBEE5gTPSJBktzTro5VigV");
    }
}
#[cfg(not(any(feature = "testnet", feature = "devnet")))]
pub mod config_feature {
    pub mod admin {
        solana_program::declare_id!("EjYMbwtvCjAdMB2RPu45QKPBEE5gTPSJBktzTro5VigV");
    }
}

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = AstrapeInstruction::unpack(instruction_data)?;

        match instruction {
            AstrapeInstruction::Initialize {
                interest_mint,
                collateral_mint,
                base_interest_rate,
                pyth_price_max_age,
                min_commission_rate,
                max_commission_rate,
                min_deposit_amount,
                max_deposit_amount,
                deposit_periods,
            } => {
                msg!("Instruction: Initialize");
                Self::process_initialize(
                    program_id,
                    accounts,
                    interest_mint,
                    collateral_mint,
                    base_interest_rate,
                    pyth_price_max_age,
                    min_commission_rate,
                    max_commission_rate,
                    min_deposit_amount,
                    max_deposit_amount,
                    deposit_periods,
                )
            }
            AstrapeInstruction::AdminUpdateConfig {
                param,
                base_interest_rate,
                pyth_price_max_age,
                min_commission_rate,
                max_commission_rate,
                min_deposit_amount,
                max_deposit_amount,
                deposit_periods,
            } => {
                msg!("Instruction: AdminUpdateConfig");
                Self::process_update_config(
                    program_id,
                    accounts,
                    param,
                    base_interest_rate,
                    pyth_price_max_age,
                    min_commission_rate,
                    max_commission_rate,
                    min_deposit_amount,
                    max_deposit_amount,
                    deposit_periods,
                )
            }
            AstrapeInstruction::AdminWithdrawCollateralForInvestment => {
                msg!("Instruction: AdminWithdrawCollateralForInvestment");
                Self::process_admin_withdraw_collateral_for_investment(program_id, accounts)
            }
            AstrapeInstruction::AdminPrepareWithdrawal => {
                msg!("Instruction: AdminPrepareWithdrawal");
                Self::process_admin_prepare_withdrawal(program_id, accounts)
            }
            AstrapeInstruction::AdminDepositInterest { amount } => {
                msg!("Instruction: AdminDepositInterest");
                Self::process_admin_deposit_interest(program_id, accounts, amount)
            }
            AstrapeInstruction::AdminWithdrawInterest { amount } => {
                msg!("Instruction: AdminWithdrawInterest");
                Self::process_admin_withdraw_interest(program_id, accounts, amount)
            }
            AstrapeInstruction::DepositCollateral {
                amount,
                deposit_period,
                commission_rate,
            } => {
                msg!("Instruction: DepositCollateral");
                Self::process_deposit_collateral(
                    program_id,
                    accounts,
                    amount,
                    deposit_period,
                    commission_rate,
                )
            }
            AstrapeInstruction::RequestWithdrawalEarly => {
                msg!("Instruction: RequestWithdrawalEarly");
                Self::process_request_withdrawal_early(program_id, accounts)
            }
            AstrapeInstruction::RequestWithdrawal => {
                msg!("Instruction: RequestWithdrawal");
                Self::process_request_withdrawal(program_id, accounts)
            }
            AstrapeInstruction::WithdrawCollateral => {
                msg!("Instruction: WithdrawCollateral");
                Self::process_withdraw_collateral(program_id, accounts)
            }
        }
    }

    fn check_pda(
        name: &str,
        pda: &Pubkey,
        seeds: &[&[u8]],
        program_id: &Pubkey,
    ) -> Result<u8, AstrapeError> {
        let (expected_pda, bump) = Pubkey::find_program_address(seeds, program_id);
        if expected_pda != *pda {
            msg!(
                "Invalid PDA for {}: expected={}, actual={}",
                name,
                expected_pda,
                pda
            );
            return Err(AstrapeError::InvalidPDA(1).into());
        }
        Ok(bump)
    }

    fn check_ata(
        name: &str,
        ata: &Pubkey,
        wallet: &Pubkey,
        mint: &Pubkey,
    ) -> Result<(), AstrapeError> {
        let expected_ata = spl_associated_token_account::get_associated_token_address(wallet, mint);
        if expected_ata != *ata {
            msg!(
                "Invalid ATA for {}: expected={}, actual={}",
                name,
                expected_ata,
                ata
            );
            return Err(AstrapeError::InvalidPDA(1).into());
        }
        Ok(())
    }

    fn deserialize_price_update(
        pyth_price_feed_account: &AccountInfo,
    ) -> Result<PriceUpdateV2, AstrapeError> {
        let buf = pyth_price_feed_account.data.borrow();
        if buf.len() < PYTH_PRICE_UPDATE_DISCRIMINATOR.len() {
            return Err(AstrapeError::InvalidPythPriceFeed);
        }
        let given_disc = &buf[..PYTH_PRICE_UPDATE_DISCRIMINATOR.len()];
        if PYTH_PRICE_UPDATE_DISCRIMINATOR != given_disc {
            return Err(AstrapeError::InvalidPythPriceFeed);
        }
        let mut data: &[u8] = &buf[PYTH_PRICE_UPDATE_DISCRIMINATOR.len()..];
        let price_update = PriceUpdateV2::deserialize(&mut data)
            .map_err(|_| AstrapeError::InvalidPythPriceFeed)?;
        Ok(price_update)
    }

    pub fn calculate_interest_amount(
        amount: u64,
        price: u64,
        commission_rate: u64,
        deposit_period: u64,
        config: &AstrapeConfig,
    ) -> u64 {
        let collateral_value_in_interest = amount * price;
        let base_interest_rate = config.base_interest_rate as f64 / 1000.0;
        let deposit_period_in_years = deposit_period as f64 / SLOTS_PER_YEAR;
        let ratio_without_commission = (1000.0 - commission_rate as f64) / 1000.0;

        let interest = collateral_value_in_interest as f64
            * base_interest_rate
            * deposit_period_in_years
            * ratio_without_commission;

        interest as u64
    }

    pub fn calculate_interest_to_return(
        user_deposit: &UserDeposit,
        current_slot: u64,
    ) -> Result<u64, AstrapeError> {
        // Calculate remaining interest to be returned based on actual lock duration
        let actual_lock_duration = current_slot - user_deposit.deposit_slot;
        let total_lock_duration = user_deposit.unlock_slot - user_deposit.deposit_slot;

        let interest_to_return = match (user_deposit.interest_received as u128)
            .checked_mul(actual_lock_duration as u128)
        {
            Some(v) => v,
            None => return Err(AstrapeError::ArithmeticOverflow.into()),
        };

        let interest_to_return = match interest_to_return.checked_div(total_lock_duration as u128) {
            Some(v) => v,
            None => {
                return Err(AstrapeError::DivisionByZero.into());
            }
        } as u64;

        Ok(interest_to_return)
    }

    fn process_initialize(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        interest_mint: Pubkey,
        collateral_mint: Pubkey,
        base_interest_rate: u64,
        pyth_price_max_age: u64,
        min_commission_rate: u64,
        max_commission_rate: u64,
        min_deposit_amount: u64,
        max_deposit_amount: u64,
        deposit_periods: Vec<u64>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;
        let authority_account = next_account_info(account_info_iter)?;
        let interest_pool_account = next_account_info(account_info_iter)?;
        let collateral_pool_account = next_account_info(account_info_iter)?;
        let withdrawal_pool_account = next_account_info(account_info_iter)?;
        let interest_mint_account = next_account_info(account_info_iter)?;
        let collateral_mint_account = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        let ata_program_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;

        // Verify admin
        if !admin_info.is_signer || config_feature::admin::id() != *admin_info.key {
            msg!("Admin must be a signer");
            return Err(AstrapeError::SignerRequired.into());
        }

        // Verify PDAs
        // 1. authority PDA
        let authority_bump = Self::check_pda(
            "authority",
            authority_account.key,
            &[AUTHORITY_SEED],
            program_id,
        )?;

        // 2. config PDA
        let config_bump = Self::check_pda("config", config_info.key, &[CONFIG_SEED], program_id)?;

        // 3. interest pool PDA
        Self::check_ata(
            "interest pool",
            interest_pool_account.key,
            authority_account.key,
            &interest_mint,
        )?;

        // 4. collateral pool PDA
        Self::check_ata(
            "collateral pool",
            collateral_pool_account.key,
            authority_account.key,
            &collateral_mint,
        )?;

        // 5. withdrawal pool PDA
        let withdrawal_pool_bump = Self::check_pda(
            "withdrawal pool",
            withdrawal_pool_account.key,
            &[WITHDRAWAL_POOL_SEED],
            program_id,
        )?;

        // Verify programs
        // 1. system program
        if *system_program_info.key != solana_program::system_program::id() {
            msg!(
                "Invalid system program: expected={}, actual={}",
                solana_program::system_program::id(),
                system_program_info.key
            );
            return Err(AstrapeError::InvalidAccountOwner.into());
        }

        // 2. token program
        if *token_program_info.key != spl_token::id() {
            msg!(
                "Invalid token program: expected={}, actual={}",
                spl_token::id(),
                token_program_info.key
            );
            return Err(AstrapeError::InvalidAccountOwner.into());
        }

        // 3. ATA program
        if *ata_program_info.key != spl_associated_token_account::id() {
            msg!(
                "Invalid ATA program: expected={}, actual={}",
                spl_associated_token_account::id(),
                ata_program_info.key
            );
            return Err(AstrapeError::InvalidAccountOwner.into());
        }

        // Check if accounts are already initialized
        if config_info.owner != system_program_info.key {
            return Err(AstrapeError::AccountAlreadyInitialized.into());
        }

        // Validate configuration parameters
        if min_commission_rate > max_commission_rate {
            return Err(AstrapeError::InvalidInput.into());
        }
        if min_deposit_amount > max_deposit_amount {
            return Err(AstrapeError::InvalidInput.into());
        }
        if deposit_periods.is_empty() {
            return Err(AstrapeError::InvalidInput.into());
        }
        if pyth_price_max_age == 0 {
            return Err(AstrapeError::ValueOutOfRange(0).into());
        }

        let rent = Rent::get()?;
        // Initialize authority account
        let authority_signer_seeds: &[&[_]] = &[AUTHORITY_SEED, &[authority_bump]];
        invoke_signed(
            &system_instruction::create_account(
                admin_info.key,
                authority_account.key,
                rent.minimum_balance(0),
                0,
                program_id,
            ),
            &[
                admin_info.clone(),
                authority_account.clone(),
                system_program_info.clone(),
            ],
            &[authority_signer_seeds],
        )?;

        // Initialize config account
        let config_signer_seeds: &[&[_]] = &[CONFIG_SEED, &[config_bump]];
        let config_size = AstrapeConfig::LEN;
        msg!("Config size: {}", config_size);
        let config_lamports = rent.minimum_balance(config_size).max(1);

        invoke_signed(
            &system_instruction::create_account(
                admin_info.key,
                config_info.key,
                config_lamports,
                config_size as u64,
                program_id,
            ),
            &[
                admin_info.clone(),
                config_info.clone(),
                system_program_info.clone(),
            ],
            &[config_signer_seeds],
        )?;

        // Create interest pool ATA - the ATA will be owned by the authority PDA
        invoke(
            &ata_instruction::create_associated_token_account(
                admin_info.key,        // Payer
                authority_account.key, // Owner of the new account (authority PDA)
                &interest_mint,        // Mint
                &spl_token::id(),      // Token program ID
            ),
            &[
                admin_info.clone(),            // Payer
                interest_pool_account.clone(), // Associated token account
                authority_account.clone(),     // Owner of the new account (authority PDA)
                interest_mint_account.clone(), // Mint account
                system_program_info.clone(),   // System program
                token_program_info.clone(),    // Token program
                ata_program_info.clone(),      // Associated token program
            ],
        )?;

        // Create collateral pool ATA - the ATA will be owned by the authority PDA
        invoke(
            &ata_instruction::create_associated_token_account(
                admin_info.key,        // Payer
                authority_account.key, // Owner of the new account (authority PDA)
                &collateral_mint,      // Mint
                &spl_token::id(),      // Token program ID
            ),
            &[
                admin_info.clone(),              // Payer
                collateral_pool_account.clone(), // Associated token account
                authority_account.clone(),       // Owner of the new account (authority PDA)
                collateral_mint_account.clone(), // Mint account
                system_program_info.clone(),     // System program
                token_program_info.clone(),      // Token program
                ata_program_info.clone(),        // Associated token program
            ],
        )?;

        // create withdrawal pool account and initialize token account
        let token_account_size = TokenAccount::LEN;
        let token_account_lamports = rent.minimum_balance(token_account_size).max(1);
        let withdrawal_pool_signer_seeds: &[&[_]] =
            &[WITHDRAWAL_POOL_SEED, &[withdrawal_pool_bump]];
        invoke_signed(
            &system_instruction::create_account(
                admin_info.key,
                withdrawal_pool_account.key,
                token_account_lamports,
                token_account_size as u64,
                token_program_info.key,
            ),
            &[
                admin_info.clone(),
                withdrawal_pool_account.clone(),
                authority_account.clone(),
                system_program_info.clone(),
            ],
            &[withdrawal_pool_signer_seeds],
        )?;
        invoke(
            &token_instruction::initialize_account(
                &spl_token::id(),
                withdrawal_pool_account.key,
                collateral_mint_account.key,
                authority_account.key,
            )?,
            &[
                withdrawal_pool_account.clone(),
                collateral_mint_account.clone(),
                authority_account.clone(),
                rent_account_info.clone(),
            ],
            // &[withdrawal_pool_signer_seeds],
        )?;

        // Initialize config with provided values
        let config = AstrapeConfig {
            interest_mint,
            collateral_mint,
            base_interest_rate,
            pyth_price_max_age,
            min_commission_rate,
            max_commission_rate,
            min_deposit_amount,
            max_deposit_amount,
            deposit_periods,
        };
        let mut config_data = config_info.data.borrow_mut();
        let mut dst = &mut config_data[..];
        config.serialize(&mut dst)?;

        Ok(())
    }

    fn process_update_config(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        param: u8,
        base_interest_rate: Option<u64>,
        pyth_price_max_age: Option<u64>,
        min_commission_rate: Option<u64>,
        max_commission_rate: Option<u64>,
        min_deposit_amount: Option<u64>,
        max_deposit_amount: Option<u64>,
        deposit_periods: Option<Vec<u64>>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;

        // Verify admin
        if !admin_info.is_signer || config_feature::admin::id() != *admin_info.key {
            msg!("Admin must be a signer");
            return Err(AstrapeError::SignerRequired.into());
        }

        // Verify config PDA
        let _config_bump = Self::check_pda("config", config_info.key, &[CONFIG_SEED], program_id)?;

        // Update config based on parameter
        let mut config = AstrapeConfig::try_from_slice(&config_info.data.borrow())?;
        match param {
            0 => {
                if let Some(rate) = base_interest_rate {
                    config.base_interest_rate = rate;
                    msg!("Updated base interest rate to {}", rate);
                }
            }
            1 => {
                if let Some(pyth_price_max_age) = pyth_price_max_age {
                    if pyth_price_max_age == 0 {
                        msg!("Price factor cannot be zero");
                        return Err(AstrapeError::ValueOutOfRange(0).into());
                    }
                    config.pyth_price_max_age = pyth_price_max_age;
                    msg!("Updated pyth price max age to {}", pyth_price_max_age);
                }
            }
            2 => {
                if let Some(min_rate) = min_commission_rate {
                    if min_rate > config.max_commission_rate {
                        msg!(
                            "Min commission rate {} exceeds max rate {}",
                            min_rate,
                            config.max_commission_rate
                        );
                        return Err(AstrapeError::ValueOutOfRange(min_rate).into());
                    }
                    config.min_commission_rate = min_rate;
                    msg!("Updated min commission rate to {}", min_rate);
                }
            }
            3 => {
                if let Some(max_rate) = max_commission_rate {
                    if max_rate < config.min_commission_rate {
                        msg!(
                            "Max commission rate {} is less than min rate {}",
                            max_rate,
                            config.min_commission_rate
                        );
                        return Err(AstrapeError::ValueOutOfRange(max_rate).into());
                    }
                    config.max_commission_rate = max_rate;
                    msg!("Updated max commission rate to {}", max_rate);
                }
            }
            4 => {
                if let Some(min_amount) = min_deposit_amount {
                    if min_amount > config.max_deposit_amount {
                        msg!(
                            "Min deposit amount {} exceeds max amount {}",
                            min_amount,
                            config.max_deposit_amount
                        );
                        return Err(AstrapeError::ValueOutOfRange(min_amount).into());
                    }
                    config.min_deposit_amount = min_amount;
                    msg!("Updated min deposit amount to {}", min_amount);
                }
            }
            5 => {
                if let Some(max_amount) = max_deposit_amount {
                    if max_amount < config.min_deposit_amount {
                        msg!(
                            "Max deposit amount {} is less than min amount {}",
                            max_amount,
                            config.min_deposit_amount
                        );
                        return Err(AstrapeError::ValueOutOfRange(max_amount).into());
                    }
                    config.max_deposit_amount = max_amount;
                    msg!("Updated max deposit amount to {}", max_amount);
                }
            }
            6 => {
                if let Some(periods) = deposit_periods {
                    if periods.is_empty() {
                        msg!("Deposit periods cannot be empty");
                        return Err(AstrapeError::InvalidInput.into());
                    }
                    config.deposit_periods = periods.clone();
                    msg!("Updated deposit periods to {:?}", periods);
                }
            }
            _ => {
                msg!("Invalid config parameter: {}", param);
                return Err(AstrapeError::InvalidConfigParam(param).into());
            }
        }

        let mut config_data = config_info.data.borrow_mut();
        let mut dst = &mut config_data[..];
        config.serialize(&mut dst)?;
        Ok(())
    }

    fn process_admin_withdraw_collateral_for_investment(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let admin_token_account = next_account_info(account_info_iter)?;
        let collateral_pool_account = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        let ata_program_info = next_account_info(account_info_iter)?;

        // Verify admin
        if !admin_info.is_signer || config_feature::admin::id() != *admin_info.key {
            return Err(AstrapeError::InvalidAdmin(0)).with_context("Admin must be signer");
        }

        let _ = Self::check_pda("config", config_info.key, &[CONFIG_SEED], program_id)?;
        let authority_bump = Self::check_pda(
            "authority",
            authority_info.key,
            &[AUTHORITY_SEED],
            program_id,
        )?;

        let config = AstrapeConfig::try_from_slice(&config_info.data.borrow())?;

        let _ = Self::check_ata(
            "collateral pool",
            collateral_pool_account.key,
            authority_info.key,
            &config.collateral_mint,
        )?;

        // Create admin's ATA if it doesn't exist
        if admin_token_account.data_is_empty() {
            invoke(
                &ata_instruction::create_associated_token_account(
                    admin_info.key,
                    admin_info.key,
                    &config.collateral_mint,
                    &spl_token::id(),
                ),
                &[
                    admin_info.clone(),
                    admin_token_account.clone(),
                    system_program_info.clone(),
                    token_program_info.clone(),
                    ata_program_info.clone(),
                ],
            )?;
        }

        // Get pool's collateral balance
        let amount = TokenAccount::unpack(&collateral_pool_account.data.borrow())?.amount;
        if amount == 0 {
            return Err(AstrapeError::InsufficientPoolBalance(0))
                .with_context("Insufficient pool balance");
        }

        // Transfer collateral to admin
        invoke_signed(
            &token_instruction::transfer(
                &spl_token::id(),
                collateral_pool_account.key,
                admin_token_account.key,
                authority_info.key,
                &[],
                amount,
            )?,
            &[
                authority_info.clone(),
                collateral_pool_account.clone(),
                admin_token_account.clone(),
            ],
            &[&[AUTHORITY_SEED, &[authority_bump]]],
        )?;

        Ok(())
    }

    fn process_admin_prepare_withdrawal(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;
        let admin_token_account = next_account_info(account_info_iter)?;
        let withdrawal_pool_account = next_account_info(account_info_iter)?;
        let user_info = next_account_info(account_info_iter)?;
        let user_deposit_account = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;

        // Verify admin
        // For testing purposes, just verify the signer
        if !admin_info.is_signer || config_feature::admin::id() != *admin_info.key {
            return Err(AstrapeError::InvalidAdmin(0)).with_context("Admin must be signer");
        }

        let _ = Self::check_pda("config", config_info.key, &[CONFIG_SEED], program_id)?;
        let _ = Self::check_pda(
            "withdrawal pool",
            withdrawal_pool_account.key,
            &[WITHDRAWAL_POOL_SEED],
            program_id,
        )?;
        let _ = Self::check_pda(
            "user deposit",
            user_deposit_account.key,
            &[user_info.key.as_ref()],
            program_id,
        )?;

        // Find user's deposit
        let mut deposit = UserDeposit::try_from_slice(&user_deposit_account.data.borrow())?;

        // Verify deposit state
        if deposit.state != UserDepositState::WithdrawRequested {
            let current_state = deposit.state as u8;
            let expected_state = UserDepositState::WithdrawRequested as u8;
            msg!(
                "Invalid deposit state: current={:?}, expected={:?}",
                deposit.state,
                UserDepositState::WithdrawRequested
            );
            return Err(AstrapeError::InvalidDepositState(current_state, expected_state).into());
        }

        // Transfer collateral from admin to withdrawal pool (not the main pool)
        invoke(
            &token_instruction::transfer(
                &spl_token::id(),
                admin_token_account.key,
                withdrawal_pool_account.key,
                admin_info.key,
                &[],
                deposit.amount,
            )?,
            &[
                admin_token_account.clone(),
                withdrawal_pool_account.clone(),
                admin_info.clone(),
            ],
        )?;

        // Update deposit state
        deposit.state = UserDepositState::WithdrawReady;
        let mut user_deposit_data = user_deposit_account.data.borrow_mut();
        let mut dst = &mut user_deposit_data[..];
        deposit.serialize(&mut dst)?;

        Ok(())
    }

    fn process_admin_deposit_interest(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let admin_interest_account = next_account_info(account_info_iter)?;
        let interest_pool_account = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        let ata_program_info = next_account_info(account_info_iter)?;

        // Verify admin
        // For testing purposes, just verify the signer
        if !admin_info.is_signer || config_feature::admin::id() != *admin_info.key {
            return Err(AstrapeError::InvalidAdmin(0)).with_context("Admin must be signer");
        }

        let _ = Self::check_pda("config", config_info.key, &[CONFIG_SEED], program_id)?;
        let _ = Self::check_pda(
            "authority",
            authority_info.key,
            &[AUTHORITY_SEED],
            program_id,
        )?;

        let config = AstrapeConfig::try_from_slice(&config_info.data.borrow())?;

        Self::check_ata(
            "interest pool",
            interest_pool_account.key,
            authority_info.key,
            &config.interest_mint,
        )?;
        // Transfer interest to pool
        invoke(
            &token_instruction::transfer(
                &spl_token::id(),
                admin_interest_account.key,
                interest_pool_account.key,
                &admin_info.key,
                &[],
                amount,
            )?,
            &[
                admin_info.clone(),
                interest_pool_account.clone(),
                admin_interest_account.clone(),
            ],
        )?;

        Ok(())
    }

    fn process_admin_withdraw_interest(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let admin_interest_account = next_account_info(account_info_iter)?;
        let interest_pool_account = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        let ata_program_info = next_account_info(account_info_iter)?;

        // Verify admin
        // For testing purposes, just verify the signer
        if !admin_info.is_signer || config_feature::admin::id() != *admin_info.key {
            return Err(AstrapeError::InvalidAdmin(0)).with_context("Admin must be signer");
        }

        let _ = Self::check_pda("config", config_info.key, &[CONFIG_SEED], program_id)?;
        let authority_bump = Self::check_pda(
            "authority",
            authority_info.key,
            &[AUTHORITY_SEED],
            program_id,
        )?;

        let config = AstrapeConfig::try_from_slice(&config_info.data.borrow())?;

        Self::check_ata(
            "admin interest",
            admin_interest_account.key,
            admin_info.key,
            &config.interest_mint,
        )?;
        Self::check_ata(
            "interest pool",
            interest_pool_account.key,
            authority_info.key,
            &config.interest_mint,
        )?;

        // Create admin's ATA if it doesn't exist
        if admin_interest_account.data_is_empty() {
            invoke(
                &ata_instruction::create_associated_token_account(
                    admin_info.key,
                    admin_info.key,
                    &config.interest_mint,
                    &spl_token::id(),
                ),
                &[
                    admin_info.clone(),
                    admin_interest_account.clone(),
                    system_program_info.clone(),
                    token_program_info.clone(),
                    ata_program_info.clone(),
                ],
            )?;
        }

        // Transfer interest from pool to admin
        invoke_signed(
            &token_instruction::transfer(
                &spl_token::id(),
                interest_pool_account.key,
                admin_interest_account.key,
                &authority_info.key,
                &[],
                amount,
            )?,
            &[
                authority_info.clone(),
                interest_pool_account.clone(),
                admin_interest_account.clone(),
            ],
            &[&[AUTHORITY_SEED, &[authority_bump]]],
        )?;

        Ok(())
    }

    fn process_deposit_collateral(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        deposit_period: u64,
        commission_rate: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let user_token_account = next_account_info(account_info_iter)?;
        let user_deposit_account = next_account_info(account_info_iter)?;
        let collateral_pool_account = next_account_info(account_info_iter)?;
        let user_interest_account = next_account_info(account_info_iter)?;
        let interest_pool_account = next_account_info(account_info_iter)?;
        let pyth_price_feed_account = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;

        let _ = Self::check_pda("config", config_info.key, &[CONFIG_SEED], program_id)?;
        let authority_bump = Self::check_pda(
            "authority",
            authority_info.key,
            &[AUTHORITY_SEED],
            program_id,
        )?;
        let user_deposit_bump = Self::check_pda(
            "user deposit",
            user_deposit_account.key,
            &[user_info.key.as_ref()],
            program_id,
        )?;

        let config = AstrapeConfig::try_from_slice(&config_info.data.borrow())?;

        Self::check_ata(
            "interest pool",
            interest_pool_account.key,
            authority_info.key,
            &config.interest_mint,
        )?;

        Self::check_ata(
            "collateral pool",
            collateral_pool_account.key,
            authority_info.key,
            &config.collateral_mint,
        )?;

        if user_deposit_account.data_is_empty() {
            let rent = Rent::get()?;
            let size = UserDeposit::LEN;
            let lamports = rent.minimum_balance(size).max(1);
            invoke_signed(
                &system_instruction::create_account(
                    user_info.key,
                    user_deposit_account.key,
                    lamports,
                    size as u64,
                    program_id,
                ),
                &[user_info.clone(), user_deposit_account.clone()],
                &[&[user_info.key.as_ref(), &[user_deposit_bump]]],
            )?;
        } else {
            return Err(AstrapeError::UserDepositAlreadyExists.into());
        }

        let clock = Clock::get()?;

        // Verify deposit amount is within limits
        if amount < config.min_deposit_amount || amount > config.max_deposit_amount {
            msg!(
                "Deposit amount out of bounds: amount={}, min={}, max={}",
                amount,
                config.min_deposit_amount,
                config.max_deposit_amount
            );
            return Err(AstrapeError::DepositAmountOutOfBounds(amount).into());
        }
        if commission_rate < config.min_commission_rate
            || commission_rate > config.max_commission_rate
        {
            msg!(
                "Commission rate out of bounds: rate={}, min={}, max={}",
                commission_rate,
                config.min_commission_rate,
                config.max_commission_rate
            );
            return Err(AstrapeError::CommissionRateOutOfBounds(commission_rate).into());
        }

        // Verify lock period is valid
        if !config.deposit_periods.contains(&deposit_period) {
            msg!(
                "Invalid lock period: period={}, allowed periods={:?}",
                deposit_period,
                config.deposit_periods
            );
            return Err(AstrapeError::InvalidLockPeriod(deposit_period).into());
        }

        let price_update = Self::deserialize_price_update(pyth_price_feed_account)?;

        let feed_id = get_feed_id_from_hex(
            "0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43",
        )
        .map_err(|_| AstrapeError::GetPriceError)?;

        let price_object = price_update
            .get_price_no_older_than(&Clock::get()?, config.pyth_price_max_age, &feed_id)
            .map_err(|_| AstrapeError::GetPriceError)?;

        let price = price_object.price as u64 * (10_u64.pow(price_object.exponent as u32));

        let interest_amount = Self::calculate_interest_amount(
            amount,
            price,
            commission_rate,
            deposit_period,
            &config,
        );

        // Transfer collateral to pool
        invoke(
            &token_instruction::transfer(
                &spl_token::id(),
                user_token_account.key,
                collateral_pool_account.key,
                &user_info.key,
                &[],
                amount,
            )?,
            &[
                user_info.clone(),
                collateral_pool_account.clone(),
                user_token_account.clone(),
            ],
        )?;

        // Transfer interest to user
        invoke_signed(
            &token_instruction::transfer(
                &spl_token::id(),
                interest_pool_account.key,
                user_interest_account.key,
                &authority_info.key,
                &[],
                interest_amount,
            )?,
            &[
                authority_info.clone(),
                interest_pool_account.clone(),
                user_interest_account.clone(),
            ],
            &[&[AUTHORITY_SEED, &[authority_bump]]],
        )?;

        // Add user deposit
        let user_deposit = UserDeposit {
            amount,
            deposit_slot: clock.slot,
            unlock_slot: clock.slot + deposit_period,
            interest_received: interest_amount,
            state: UserDepositState::Deposited,
            commission_rate,
        };
        let mut user_deposit_data = user_deposit_account.data.borrow_mut();
        let mut dst = &mut user_deposit_data[..];
        user_deposit.serialize(&mut dst)?;

        Ok(())
    }

    fn process_request_withdrawal_early(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let user_deposit_account = next_account_info(account_info_iter)?;
        let user_interest_account = next_account_info(account_info_iter)?;
        let interest_pool_account = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;

        let _ = Self::check_pda("config", config_info.key, &[CONFIG_SEED], program_id)?;
        let _ = Self::check_pda(
            "authority",
            authority_info.key,
            &[AUTHORITY_SEED],
            program_id,
        )?;
        let _ = Self::check_pda(
            "user deposit",
            user_deposit_account.key,
            &[user_info.key.as_ref()],
            program_id,
        )?;

        let config = AstrapeConfig::try_from_slice(&config_info.data.borrow())?;

        Self::check_ata(
            "interest pool",
            interest_pool_account.key,
            authority_info.key,
            &config.interest_mint,
        )?;

        let clock = Clock::get()?;
        // Find user's deposit
        let mut deposit = UserDeposit::try_from_slice(&user_deposit_account.data.borrow())?;

        // Verify deposit state
        if deposit.state != UserDepositState::Deposited {
            let current_state = deposit.state as u8;
            let expected_state = UserDepositState::Deposited as u8;
            msg!(
                "Invalid deposit state: current={:?}, expected={:?}",
                deposit.state,
                UserDepositState::Deposited
            );
            return Err(AstrapeError::InvalidDepositState(current_state, expected_state).into());
        }

        let interest_to_return = Self::calculate_interest_to_return(&deposit, clock.slot)?;

        // Check if user has enough interest tokens to return
        let user_interest_balance =
            TokenAccount::unpack(&user_interest_account.data.borrow())?.amount;
        if user_interest_balance < interest_to_return {
            msg!(
                "Insufficient interest balance: balance={}, required={}",
                user_interest_balance,
                interest_to_return
            );
            return Err(AstrapeError::InsufficientInterestBalance(user_interest_balance).into());
        }

        // Transfer interest back to pool
        invoke(
            &token_instruction::transfer(
                &spl_token::id(),
                user_interest_account.key,
                interest_pool_account.key,
                &user_info.key,
                &[],
                interest_to_return,
            )?,
            &[
                user_info.clone(),
                interest_pool_account.clone(),
                user_interest_account.clone(),
            ],
        )?;

        // Update deposit state
        deposit.state = UserDepositState::WithdrawRequested;
        let mut user_deposit_data = user_deposit_account.data.borrow_mut();
        let mut dst = &mut user_deposit_data[..];
        deposit.serialize(&mut dst)?;

        Ok(())
    }

    fn process_request_withdrawal(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user_info = next_account_info(account_info_iter)?;
        let user_deposit_account = next_account_info(account_info_iter)?;

        let _ = Self::check_pda(
            "user deposit",
            user_deposit_account.key,
            &[user_info.key.as_ref()],
            program_id,
        )?;

        let clock = Clock::get()?;
        let mut deposit = UserDeposit::try_from_slice(&user_deposit_account.data.borrow())?;

        if deposit.state != UserDepositState::WithdrawRequested {
            let current_state = deposit.state as u8;
            let expected_state = UserDepositState::WithdrawRequested as u8;
            msg!(
                "Invalid deposit state: current={:?}, expected={:?}",
                deposit.state,
                UserDepositState::WithdrawRequested
            );

            return Err(AstrapeError::InvalidDepositState(current_state, expected_state).into());
        }

        if clock.slot < deposit.unlock_slot {
            msg!(
                "Deposit is not yet unlocked: slot={}, unlock_slot={}",
                clock.slot,
                deposit.unlock_slot
            );
            return Err(AstrapeError::NotUnlockedYet(clock.slot, deposit.unlock_slot).into());
        }

        deposit.state = UserDepositState::WithdrawReady;
        let mut user_deposit_data = user_deposit_account.data.borrow_mut();
        let mut dst = &mut user_deposit_data[..];
        deposit.serialize(&mut dst)?;

        Ok(())
    }

    fn process_withdraw_collateral(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let user_deposit_account = next_account_info(account_info_iter)?;
        let user_token_account = next_account_info(account_info_iter)?;
        let withdrawal_pool_account = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;

        let _ = Self::check_pda("config", config_info.key, &[CONFIG_SEED], program_id)?;
        let authority_bump = Self::check_pda(
            "authority",
            authority_info.key,
            &[AUTHORITY_SEED],
            program_id,
        )?;
        let _ = Self::check_pda(
            "user deposit",
            user_deposit_account.key,
            &[user_info.key.as_ref()],
            program_id,
        )?;
        let _ = Self::check_pda(
            "withdrawal pool",
            withdrawal_pool_account.key,
            &[WITHDRAWAL_POOL_SEED],
            program_id,
        )?;

        let _config = AstrapeConfig::try_from_slice(&config_info.data.borrow())?;
        // Find user's deposit
        let mut deposit = UserDeposit::try_from_slice(&user_deposit_account.data.borrow())?;

        // Verify deposit state
        if deposit.state != UserDepositState::WithdrawReady {
            let current_state = deposit.state as u8;
            let expected_state = UserDepositState::WithdrawReady as u8;
            msg!(
                "Invalid deposit state: current={:?}, expected={:?}",
                deposit.state,
                UserDepositState::WithdrawReady
            );
            return Err(AstrapeError::InvalidDepositState(current_state, expected_state).into());
        }

        // Check if withdrawal pool has enough tokens
        let pool_balance = TokenAccount::unpack(&withdrawal_pool_account.data.borrow())?.amount;
        if pool_balance < deposit.amount {
            msg!(
                "Insufficient withdrawal pool balance: balance={}, required={}",
                pool_balance,
                deposit.amount
            );
            return Err(AstrapeError::InsufficientPoolBalance(pool_balance).into());
        }

        // Transfer collateral from withdrawal pool to user
        invoke_signed(
            &token_instruction::transfer(
                &spl_token::id(),
                withdrawal_pool_account.key,
                user_token_account.key,
                &authority_info.key,
                &[],
                deposit.amount,
            )?,
            &[
                authority_info.clone(),
                withdrawal_pool_account.clone(),
                user_token_account.clone(),
            ],
            &[&[AUTHORITY_SEED, &[authority_bump]]],
        )?;

        // Remove deposit
        deposit.state = UserDepositState::WithdrawCompleted;
        let mut user_deposit_data = user_deposit_account.data.borrow_mut();
        let mut dst = &mut user_deposit_data[..];
        deposit.serialize(&mut dst)?;

        Ok(())
    }
}
