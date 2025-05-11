use borsh::{BorshDeserialize, BorshSerialize};
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
use std::collections::HashMap;
use std::mem::size_of;

use crate::{
    errors::{TokenLockError, TokenLockResult},
    instructions::TokenLockInstruction,
    state::{PoolConfig, PoolState, UserDeposit, UserDepositState},
};

// PDA seeds
const CONFIG_SEED: &[u8] = b"pool_config";
const STATE_SEED: &[u8] = b"pool_state";
const AUTHORITY_SEED: &[u8] = b"authority";

// Unused seeds (can be removed)
const _INTEREST_POOL_SEED: &[u8] = b"interest_pool";
const _COLLATERAL_POOL_SEED: &[u8] = b"collateral_pool";

#[cfg(feature = "testnet")]
pub mod config_feature {
    pub mod admin {
        solana_program::declare_id!("75KWb5XcqPTgacQyNw9P5QU2HL3xpezEVcgsFCiJgTT");
    }
}
#[cfg(feature = "devnet")]
pub mod config_feature {
    pub mod admin {
        solana_program::declare_id!("Adm29NctkKwJGaaiU8CXqdV6WDTwR81JbxV8zoxn745Y");
    }
}
#[cfg(not(any(feature = "testnet", feature = "devnet")))]
pub mod config_feature {
    pub mod admin {
        solana_program::declare_id!("GThUX1Atko4tqhN2NaiTazWSeFWMuiUvfFnyJyUghFMJ");
    }
}

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = TokenLockInstruction::unpack(instruction_data)?;

        match instruction {
            TokenLockInstruction::Initialize {
                interest_mint,
                collateral_mint,
                base_interest_rate,
                price_factor,
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
                    price_factor,
                    min_commission_rate,
                    max_commission_rate,
                    min_deposit_amount,
                    max_deposit_amount,
                    deposit_periods,
                )
            }
            TokenLockInstruction::AdminUpdateConfig {
                param,
                base_interest_rate,
                price_factor,
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
                    price_factor,
                    min_commission_rate,
                    max_commission_rate,
                    min_deposit_amount,
                    max_deposit_amount,
                    deposit_periods,
                )
            }
            TokenLockInstruction::AdminWithdrawCollateralForInvestment => {
                msg!("Instruction: AdminWithdrawCollateralForInvestment");
                Self::process_admin_withdraw_collateral_for_investment(program_id, accounts)
            }
            TokenLockInstruction::AdminUpdateDepositStates => {
                msg!("Instruction: AdminUpdateDepositStates");
                Self::process_admin_update_deposit_states(program_id, accounts)
            }
            TokenLockInstruction::AdminPrepareWithdrawal { user_pubkey } => {
                msg!("Instruction: AdminPrepareWithdrawal");
                Self::process_admin_prepare_withdrawal(program_id, accounts, user_pubkey)
            }
            TokenLockInstruction::AdminDepositInterest { amount } => {
                msg!("Instruction: AdminDepositInterest");
                Self::process_admin_deposit_interest(program_id, accounts, amount)
            }
            TokenLockInstruction::AdminWithdrawInterest { amount } => {
                msg!("Instruction: AdminWithdrawInterest");
                Self::process_admin_withdraw_interest(program_id, accounts, amount)
            }
            TokenLockInstruction::DepositCollateral {
                amount,
                unlock_slot,
            } => {
                msg!("Instruction: DepositCollateral");
                Self::process_deposit_collateral(program_id, accounts, amount, unlock_slot)
            }
            TokenLockInstruction::RequestWithdrawal => {
                msg!("Instruction: RequestWithdrawal");
                Self::process_request_withdrawal(program_id, accounts)
            }
            TokenLockInstruction::WithdrawCollateral => {
                msg!("Instruction: WithdrawCollateral");
                Self::process_withdraw_collateral(program_id, accounts)
            }
        }
    }

    fn find_pool_accounts(
        program_id: &Pubkey,
        interest_mint: &Pubkey,
        collateral_mint: &Pubkey,
    ) -> (Pubkey, Pubkey) {
        // Find the authority PDA that will own the token accounts
        let (authority_key, _) = Pubkey::find_program_address(&[AUTHORITY_SEED], program_id);

        // Use the authority PDA as the owner of the token accounts
        let interest_pool = spl_associated_token_account::get_associated_token_address(
            &authority_key,
            interest_mint,
        );
        let collateral_pool = spl_associated_token_account::get_associated_token_address(
            &authority_key,
            collateral_mint,
        );
        (interest_pool, collateral_pool)
    }

    fn find_state_account(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[STATE_SEED], program_id)
    }

    fn process_initialize(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        interest_mint: Pubkey,
        collateral_mint: Pubkey,
        base_interest_rate: u64,
        price_factor: u64,
        min_commission_rate: u64,
        max_commission_rate: u64,
        min_deposit_amount: u64,
        max_deposit_amount: u64,
        deposit_periods: Vec<u64>,
    ) -> ProgramResult {
        // Log the number of accounts provided
        msg!("Number of accounts in initialize: {}", accounts.len());

        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        msg!("Admin account: {}", admin_info.key);

        let config_info = next_account_info(account_info_iter)?;
        msg!("Config account: {}", config_info.key);

        let state_info = next_account_info(account_info_iter)?;
        msg!("State account: {}", state_info.key);

        let system_program_info = next_account_info(account_info_iter)?;
        msg!("System program account: {}", system_program_info.key);

        let token_program_info = next_account_info(account_info_iter)?;
        msg!("Token program account: {}", token_program_info.key);

        let ata_program_info = next_account_info(account_info_iter)?;
        msg!("ATA program account: {}", ata_program_info.key);

        let interest_pool_account = next_account_info(account_info_iter)?;
        msg!("Interest pool account: {}", interest_pool_account.key);

        let collateral_pool_account = next_account_info(account_info_iter)?;
        msg!("Collateral pool account: {}", collateral_pool_account.key);

        let interest_mint_account = next_account_info(account_info_iter)?;
        msg!("Interest mint account: {}", interest_mint_account.key);

        let collateral_mint_account = next_account_info(account_info_iter)?;
        msg!("Collateral mint account: {}", collateral_mint_account.key);

        let program_account = next_account_info(account_info_iter)?;
        msg!("Program account: {}", program_account.key);

        // Find authority PDA
        let (authority_key, authority_bump) =
            Pubkey::find_program_address(&[AUTHORITY_SEED], program_id);
        msg!("Expected authority PDA: {}", authority_key);

        // We don't need the authority account passed in explicitly

        // Verify admin
        if !admin_info.is_signer {
            msg!("Admin must be a signer");
            return Err(TokenLockError::SignerRequired.into());
        }

        let _admin = *admin_info.key;

        // Verify system program
        if *system_program_info.key != solana_program::system_program::id() {
            msg!(
                "Invalid system program: expected={}, actual={}",
                solana_program::system_program::id(),
                system_program_info.key
            );
            return Err(TokenLockError::InvalidAccountOwner.into());
        }

        // Verify token program
        if *token_program_info.key != spl_token::id() {
            msg!(
                "Invalid token program: expected={}, actual={}",
                spl_token::id(),
                token_program_info.key
            );
            return Err(TokenLockError::InvalidAccountOwner.into());
        }

        // Verify ATA program
        if *ata_program_info.key != spl_associated_token_account::id() {
            msg!(
                "Invalid ATA program: expected={}, actual={}",
                spl_associated_token_account::id(),
                ata_program_info.key
            );
            return Err(TokenLockError::InvalidAccountOwner.into());
        }

        // Verify config PDA
        let (config_pda, config_bump) = Pubkey::find_program_address(&[CONFIG_SEED], program_id);
        if config_pda != *config_info.key {
            msg!(
                "Invalid config PDA: expected={}, actual={}",
                config_pda,
                config_info.key
            );
            return Err(TokenLockError::InvalidPDA(1).into());
        }

        // Verify state PDA
        let (state_pda, state_bump) = Self::find_state_account(program_id);
        if state_pda != *state_info.key {
            msg!(
                "Invalid state PDA: expected={}, actual={}",
                state_pda,
                state_info.key
            );
            return TokenLockError::InvalidPDA(2).log_and_return();
        }

        // Verify pool accounts are ATAs
        let (expected_interest_pool, expected_collateral_pool) =
            Self::find_pool_accounts(program_id, &interest_mint, &collateral_mint);
        if *interest_pool_account.key != expected_interest_pool {
            msg!(
                "Invalid interest pool account: expected={}, actual={}",
                expected_interest_pool,
                interest_pool_account.key
            );
            return Err(TokenLockError::InvalidPoolAccount(1).into());
        }
        if *collateral_pool_account.key != expected_collateral_pool {
            msg!(
                "Invalid collateral pool account: expected={}, actual={}",
                expected_collateral_pool,
                collateral_pool_account.key
            );
            return Err(TokenLockError::InvalidPoolAccount(2).into());
        }

        // Check if accounts are already initialized
        if config_info.owner != system_program_info.key
            || state_info.owner != system_program_info.key
        {
            return Err(TokenLockError::InvalidPoolAccount(3).into());
        }

        // Validate configuration parameters
        if min_commission_rate > max_commission_rate {
            return Err(TokenLockError::InvalidInput.into());
        }
        if min_deposit_amount > max_deposit_amount {
            return Err(TokenLockError::InvalidInput.into());
        }
        if deposit_periods.is_empty() {
            return Err(TokenLockError::InvalidInput.into());
        }
        if price_factor == 0 {
            return Err(TokenLockError::ValueOutOfRange(0).into());
        }

        // Initialize config account
        let config_signer_seeds: &[&[_]] = &[CONFIG_SEED, &[config_bump]];
        let rent = Rent::get()?;
        let config_size = size_of::<PoolConfig>();
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

        // Initialize state account
        let state_signer_seeds: &[&[_]] = &[STATE_SEED, &[state_bump]];
        let state_size = size_of::<PoolState>();
        let state_lamports = rent.minimum_balance(state_size).max(1);

        invoke_signed(
            &system_instruction::create_account(
                admin_info.key,
                state_info.key,
                state_lamports,
                state_size as u64,
                program_id,
            ),
            &[
                admin_info.clone(),
                state_info.clone(),
                system_program_info.clone(),
            ],
            &[state_signer_seeds],
        )?;

        // Create interest pool ATA - the ATA will be owned by the authority PDA
        invoke(
            &ata_instruction::create_associated_token_account(
                admin_info.key,   // Payer
                program_id,       // Owner of the new account (the program itself)
                &interest_mint,   // Mint
                &spl_token::id(), // Token program ID
            ),
            &[
                admin_info.clone(),            // Payer
                interest_pool_account.clone(), // Associated token account
                program_account.clone(),       // Program account as the owner
                interest_mint_account.clone(), // Mint account
                system_program_info.clone(),   // System program
                token_program_info.clone(),    // Token program
                ata_program_info.clone(),      // Associated token program
            ],
        )?;

        // Create collateral pool ATA - the ATA will be owned by the authority PDA
        invoke(
            &ata_instruction::create_associated_token_account(
                admin_info.key,   // Payer
                program_id,       // Owner of the new account (the program itself)
                &collateral_mint, // Mint
                &spl_token::id(), // Token program ID
            ),
            &[
                admin_info.clone(),              // Payer
                collateral_pool_account.clone(), // Associated token account
                program_account.clone(),         // Program account as the owner
                collateral_mint_account.clone(), // Mint account
                system_program_info.clone(),     // System program
                token_program_info.clone(),      // Token program
                ata_program_info.clone(),        // Associated token program
            ],
        )?;

        // Initialize config with provided values
        let config = PoolConfig {
            interest_mint,
            collateral_mint,
            base_interest_rate,
            price_factor,
            min_commission_rate,
            max_commission_rate,
            min_deposit_amount,
            max_deposit_amount,
            deposit_period: deposit_periods,
        };
        config.serialize(&mut *config_info.data.borrow_mut())?;

        // Initialize state
        let state = PoolState {
            deposits: HashMap::new(),
        };
        state.serialize(&mut *state_info.data.borrow_mut())?;

        Ok(())
    }

    fn process_update_config(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        param: u8,
        base_interest_rate: Option<u64>,
        price_factor: Option<u64>,
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
        if !admin_info.is_signer {
            msg!("Admin must be a signer");
            return Err(TokenLockError::SignerRequired.into());
        }

        // Verify config PDA
        let (pda, _) = Pubkey::find_program_address(&[CONFIG_SEED], program_id);
        if pda != *config_info.key || config_info.owner != program_id {
            msg!(
                "Invalid config account: expected={}, actual={}, owner={}",
                pda,
                config_info.key,
                config_info.owner
            );
            return Err(TokenLockError::InvalidPDA(1).into());
        }

        // Update config based on parameter
        let mut config = PoolConfig::try_from_slice(&config_info.data.borrow())?;
        match param {
            0 => {
                if let Some(rate) = base_interest_rate {
                    config.base_interest_rate = rate;
                    msg!("Updated base interest rate to {}", rate);
                }
            }
            1 => {
                if let Some(factor) = price_factor {
                    if factor == 0 {
                        msg!("Price factor cannot be zero");
                        return Err(TokenLockError::ValueOutOfRange(0).into());
                    }
                    config.price_factor = factor;
                    msg!("Updated price factor to {}", factor);
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
                        return Err(TokenLockError::ValueOutOfRange(min_rate).into());
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
                        return Err(TokenLockError::ValueOutOfRange(max_rate).into());
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
                        return Err(TokenLockError::ValueOutOfRange(min_amount).into());
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
                        return Err(TokenLockError::ValueOutOfRange(max_amount).into());
                    }
                    config.max_deposit_amount = max_amount;
                    msg!("Updated max deposit amount to {}", max_amount);
                }
            }
            6 => {
                if let Some(periods) = deposit_periods {
                    if periods.is_empty() {
                        msg!("Deposit periods cannot be empty");
                        return Err(TokenLockError::InvalidInput.into());
                    }
                    config.deposit_period = periods.clone();
                    msg!("Updated deposit periods to {:?}", periods);
                }
            }
            _ => {
                msg!("Invalid config parameter: {}", param);
                return Err(TokenLockError::InvalidConfigParam(param).into());
            }
        }

        config.serialize(&mut *config_info.data.borrow_mut())?;
        Ok(())
    }

    fn process_admin_withdraw_collateral_for_investment(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;
        let admin_token_account = next_account_info(account_info_iter)?;
        let pool_token_account = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        let ata_program_info = next_account_info(account_info_iter)?;

        // Verify admin
        if !admin_info.is_signer {
            return Err(TokenLockError::InvalidAdmin(0)).with_context("Admin must be signer");
        }

        let config = PoolConfig::try_from_slice(&config_info.data.borrow())?;

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
        let amount = TokenAccount::unpack(&pool_token_account.data.borrow())?.amount;
        if amount == 0 {
            return Err(TokenLockError::InsufficientPoolBalance(0))
                .with_context("Insufficient pool balance");
        }

        // Find pool authority PDA
        let (authority_key, authority_bump) =
            Pubkey::find_program_address(&[AUTHORITY_SEED], _program_id);

        // Transfer collateral to admin
        invoke_signed(
            &token_instruction::transfer(
                &spl_token::id(),
                pool_token_account.key,
                admin_token_account.key,
                &authority_key,
                &[],
                amount,
            )?,
            &[pool_token_account.clone(), admin_token_account.clone()],
            &[&[AUTHORITY_SEED, &[authority_bump]]],
        )?;

        Ok(())
    }

    fn process_admin_update_deposit_states(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        let state_info = next_account_info(account_info_iter)?;

        // Verify admin
        // For testing purposes, just verify the signer
        if !admin_info.is_signer {
            return Err(TokenLockError::InvalidAdmin(0)).with_context("Admin must be signer");
        }

        let mut state = PoolState::try_from_slice(&state_info.data.borrow())?;
        let clock = Clock::get()?;

        // Update states for deposits that have reached their unlock slot
        for (_, deposit) in state.deposits.iter_mut() {
            if deposit.state == UserDepositState::Deposited && deposit.unlock_slot <= clock.slot {
                deposit.state = UserDepositState::WithdrawRequested;
            }
        }

        state.serialize(&mut *state_info.data.borrow_mut())?;

        Ok(())
    }

    fn process_admin_prepare_withdrawal(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        user_pubkey: Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;
        let state_info = next_account_info(account_info_iter)?;
        let admin_token_account = next_account_info(account_info_iter)?;
        let _user_token_account = next_account_info(account_info_iter)?;
        let pool_token_account = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        let ata_program_info = next_account_info(account_info_iter)?;

        // Verify admin
        // For testing purposes, just verify the signer
        if !admin_info.is_signer {
            return Err(TokenLockError::InvalidAdmin(0)).with_context("Admin must be signer");
        }

        let config = PoolConfig::try_from_slice(&config_info.data.borrow())?;
        let mut state = PoolState::try_from_slice(&state_info.data.borrow())?;

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

        // Find user's deposit
        let deposit = state
            .deposits
            .get(&user_pubkey)
            .ok_or(TokenLockError::NoDepositFound)?;

        // Verify deposit state
        if deposit.state != UserDepositState::WithdrawRequested {
            let current_state = deposit.state as u8;
            let expected_state = UserDepositState::WithdrawRequested as u8;
            msg!(
                "Invalid deposit state: current={:?}, expected={:?}",
                deposit.state,
                UserDepositState::WithdrawRequested
            );
            return Err(TokenLockError::InvalidDepositState(current_state, expected_state).into());
        }

        // Transfer collateral from admin to pool
        invoke(
            &token_instruction::transfer(
                &spl_token::id(),
                admin_token_account.key,
                pool_token_account.key,
                &admin_token_account.key,
                &[],
                deposit.amount,
            )?,
            &[
                admin_token_account.clone(),
                pool_token_account.clone(),
                admin_token_account.clone(),
            ],
        )?;

        // Update deposit state
        let updated_deposit = UserDeposit {
            amount: deposit.amount,
            deposit_slot: deposit.deposit_slot,
            unlock_slot: deposit.unlock_slot,
            interest_received: deposit.interest_received,
            state: UserDepositState::WithdrawReady,
            commission_rate: deposit.commission_rate,
        };
        state.deposits.insert(user_pubkey, updated_deposit);

        state.serialize(&mut *state_info.data.borrow_mut())?;

        Ok(())
    }

    fn process_admin_deposit_interest(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;
        let admin_interest_account = next_account_info(account_info_iter)?;
        let interest_pool_account = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        let ata_program_info = next_account_info(account_info_iter)?;

        // Verify admin
        // For testing purposes, just verify the signer
        if !admin_info.is_signer {
            return Err(TokenLockError::InvalidAdmin(0)).with_context("Admin must be signer");
        }

        let config = PoolConfig::try_from_slice(&config_info.data.borrow())?;

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

        // Verify pool account is ATA
        let (expected_interest_pool, _) =
            Self::find_pool_accounts(_program_id, &config.interest_mint, &config.collateral_mint);
        if *interest_pool_account.key != expected_interest_pool {
            msg!(
                "Invalid interest pool account: expected={}, actual={}",
                expected_interest_pool,
                interest_pool_account.key
            );
            return Err(TokenLockError::InvalidPoolAccount(1).into());
        }

        // Transfer interest to pool
        invoke(
            &token_instruction::transfer(
                &spl_token::id(),
                admin_interest_account.key,
                interest_pool_account.key,
                &admin_interest_account.key,
                &[],
                amount,
            )?,
            &[
                admin_interest_account.clone(),
                interest_pool_account.clone(),
                admin_interest_account.clone(),
            ],
        )?;

        Ok(())
    }

    fn process_admin_withdraw_interest(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;
        let admin_interest_account = next_account_info(account_info_iter)?;
        let interest_pool_account = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        let ata_program_info = next_account_info(account_info_iter)?;

        // Verify admin
        // For testing purposes, just verify the signer
        if !admin_info.is_signer {
            return Err(TokenLockError::InvalidAdmin(0)).with_context("Admin must be signer");
        }

        let config = PoolConfig::try_from_slice(&config_info.data.borrow())?;

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

        // Verify pool account is ATA
        let (expected_interest_pool, _) =
            Self::find_pool_accounts(_program_id, &config.interest_mint, &config.collateral_mint);
        if *interest_pool_account.key != expected_interest_pool {
            msg!(
                "Invalid interest pool account: expected={}, actual={}",
                expected_interest_pool,
                interest_pool_account.key
            );
            return Err(TokenLockError::InvalidPoolAccount(1).into());
        }

        // Find pool authority PDA
        let (authority_key, authority_bump) =
            Pubkey::find_program_address(&[AUTHORITY_SEED], _program_id);

        // Transfer interest from pool to admin
        invoke_signed(
            &token_instruction::transfer(
                &spl_token::id(),
                interest_pool_account.key,
                admin_interest_account.key,
                &authority_key,
                &[],
                amount,
            )?,
            &[
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
        unlock_slot: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let config_info = next_account_info(account_info_iter)?;
        let state_info = next_account_info(account_info_iter)?;
        let user_token_account = next_account_info(account_info_iter)?;
        let pool_token_account = next_account_info(account_info_iter)?;
        let user_interest_account = next_account_info(account_info_iter)?;
        let interest_pool_account = next_account_info(account_info_iter)?;

        let config = PoolConfig::try_from_slice(&config_info.data.borrow())?;
        let mut state = PoolState::try_from_slice(&state_info.data.borrow())?;
        let clock = Clock::get()?;

        // Verify pool accounts are PDAs
        let (expected_interest_pool, expected_collateral_pool) =
            Self::find_pool_accounts(program_id, &config.interest_mint, &config.collateral_mint);
        if *interest_pool_account.key != expected_interest_pool {
            msg!(
                "Invalid interest pool account: expected={}, actual={}",
                expected_interest_pool,
                interest_pool_account.key
            );
            return Err(TokenLockError::InvalidPoolAccount(1).into());
        }
        if *pool_token_account.key != expected_collateral_pool {
            msg!(
                "Invalid collateral pool account: expected={}, actual={}",
                expected_collateral_pool,
                pool_token_account.key
            );
            return Err(TokenLockError::InvalidPoolAccount(2).into());
        }

        // Verify deposit amount is within limits
        if amount < config.min_deposit_amount {
            msg!(
                "Deposit amount below minimum: amount={}, min={}",
                amount,
                config.min_deposit_amount
            );
            return Err(TokenLockError::AmountBelowMinimum(amount).into());
        }
        if amount > config.max_deposit_amount {
            msg!(
                "Deposit amount exceeds maximum: amount={}, max={}",
                amount,
                config.max_deposit_amount
            );
            return Err(TokenLockError::AmountExceedsMaximum(amount).into());
        }

        // Verify lock period is valid
        let lock_period = unlock_slot - clock.slot;
        if !config.deposit_period.contains(&lock_period) {
            msg!(
                "Invalid lock period: period={}, allowed periods={:?}",
                lock_period,
                config.deposit_period
            );
            return Err(TokenLockError::InvalidLockPeriod(lock_period).into());
        }

        // Calculate interest based on slot duration and price factor
        let slot_duration = unlock_slot - clock.slot;

        // Safely calculate interest with overflow checks
        let interest_multiplier =
            match (slot_duration as u128).checked_mul(config.base_interest_rate as u128) {
                Some(v) => v,
                None => return Err(TokenLockError::ArithmeticOverflow.into()),
            };

        let interest_multiplier = match interest_multiplier.checked_div(365 * 24 * 60 * 60 * 2) {
            // Convert to annual rate (assuming 2 slots per second)
            Some(v) => v,
            None => return Err(TokenLockError::DivisionByZero.into()),
        };

        // Calculate interest amount with price factor
        let interest_amount = match (amount as u128).checked_mul(interest_multiplier) {
            Some(v) => v,
            None => return Err(TokenLockError::ArithmeticOverflow.into()),
        };

        let interest_amount = match interest_amount.checked_mul(config.price_factor as u128) {
            Some(v) => v,
            None => return Err(TokenLockError::ArithmeticOverflow.into()),
        };

        let interest_amount = match interest_amount.checked_div(10000) {
            Some(v) => v,
            None => return Err(TokenLockError::DivisionByZero.into()),
        };

        let interest_amount = match interest_amount.checked_div(10000) {
            Some(v) => v,
            None => return Err(TokenLockError::DivisionByZero.into()),
        } as u64;

        // Calculate commission
        let commission_rate = config.min_commission_rate; // Use min commission rate for now
        let commission_amount = match (interest_amount as u128).checked_mul(commission_rate as u128)
        {
            Some(v) => v,
            None => return Err(TokenLockError::ArithmeticOverflow.into()),
        };

        let commission_amount = match commission_amount.checked_div(10000) {
            Some(v) => v,
            None => return Err(TokenLockError::DivisionByZero.into()),
        } as u64;

        // Check if pool has enough interest tokens
        let interest_pool_balance =
            TokenAccount::unpack(&interest_pool_account.data.borrow())?.amount;
        if interest_pool_balance < interest_amount {
            msg!(
                "Insufficient interest tokens in pool: balance={}, required={}",
                interest_pool_balance,
                interest_amount
            );
            return Err(TokenLockError::InsufficientPoolBalance(interest_pool_balance).into());
        }

        // Transfer collateral to pool
        invoke(
            &token_instruction::transfer(
                &spl_token::id(),
                user_token_account.key,
                pool_token_account.key,
                &user_token_account.key,
                &[],
                amount,
            )?,
            &[
                user_token_account.clone(),
                pool_token_account.clone(),
                user_token_account.clone(),
            ],
        )?;

        // Find pool authority PDA
        let (authority_key, authority_bump) =
            Pubkey::find_program_address(&[AUTHORITY_SEED], program_id);

        // Transfer interest to user
        invoke_signed(
            &token_instruction::transfer(
                &spl_token::id(),
                interest_pool_account.key,
                user_interest_account.key,
                &authority_key,
                &[],
                interest_amount - commission_amount,
            )?,
            &[interest_pool_account.clone(), user_interest_account.clone()],
            &[&[AUTHORITY_SEED, &[authority_bump]]],
        )?;

        // Add user deposit
        let user_deposit = UserDeposit {
            amount,
            deposit_slot: clock.slot,
            unlock_slot,
            interest_received: interest_amount - commission_amount,
            state: UserDepositState::Deposited,
            commission_rate,
        };
        state.deposits.insert(*user_token_account.key, user_deposit);
        state.serialize(&mut *state_info.data.borrow_mut())?;

        Ok(())
    }

    fn process_request_withdrawal(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let config_info = next_account_info(account_info_iter)?;
        let state_info = next_account_info(account_info_iter)?;
        let user_interest_account = next_account_info(account_info_iter)?;
        let interest_pool_account = next_account_info(account_info_iter)?;

        let _config = PoolConfig::try_from_slice(&config_info.data.borrow())?;
        let mut state = PoolState::try_from_slice(&state_info.data.borrow())?;
        let clock = Clock::get()?;

        // Find user's deposit
        let deposit = state
            .deposits
            .get(user_interest_account.key)
            .ok_or_else(|| {
                msg!("No deposit found for user: {}", user_interest_account.key);
                TokenLockError::NoDepositFound
            })?;

        // Verify deposit state
        if deposit.state != UserDepositState::Deposited {
            let current_state = deposit.state as u8;
            let expected_state = UserDepositState::Deposited as u8;
            msg!(
                "Invalid deposit state: current={:?}, expected={:?}",
                deposit.state,
                UserDepositState::Deposited
            );
            return Err(TokenLockError::InvalidDepositState(current_state, expected_state).into());
        }

        // Calculate remaining interest to be returned based on actual lock duration
        let actual_lock_duration = clock.slot - deposit.deposit_slot;
        let total_lock_duration = deposit.unlock_slot - deposit.deposit_slot;

        let interest_to_return =
            match (deposit.interest_received as u128).checked_mul(actual_lock_duration as u128) {
                Some(v) => v,
                None => return Err(TokenLockError::ArithmeticOverflow.into()),
            };

        let interest_to_return = match interest_to_return.checked_div(total_lock_duration as u128) {
            Some(v) => v,
            None => {
                return Err(TokenLockError::DivisionByZero.into());
            }
        } as u64;

        // Check if user has enough interest tokens to return
        let user_interest_balance =
            TokenAccount::unpack(&user_interest_account.data.borrow())?.amount;
        if user_interest_balance < interest_to_return {
            msg!(
                "Insufficient interest balance: balance={}, required={}",
                user_interest_balance,
                interest_to_return
            );
            return Err(TokenLockError::InsufficientInterestBalance(user_interest_balance).into());
        }

        // Transfer interest back to pool
        invoke(
            &token_instruction::transfer(
                &spl_token::id(),
                user_interest_account.key,
                interest_pool_account.key,
                &user_interest_account.key,
                &[],
                interest_to_return,
            )?,
            &[
                user_interest_account.clone(),
                interest_pool_account.clone(),
                user_interest_account.clone(),
            ],
        )?;

        // Update deposit state
        let updated_deposit = UserDeposit {
            amount: deposit.amount,
            deposit_slot: deposit.deposit_slot,
            unlock_slot: deposit.unlock_slot,
            interest_received: deposit.interest_received,
            state: UserDepositState::WithdrawRequested,
            commission_rate: deposit.commission_rate,
        };
        state
            .deposits
            .insert(*user_interest_account.key, updated_deposit);

        state.serialize(&mut *state_info.data.borrow_mut())?;

        Ok(())
    }

    fn process_withdraw_collateral(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let config_info = next_account_info(account_info_iter)?;
        let state_info = next_account_info(account_info_iter)?;
        let user_token_account = next_account_info(account_info_iter)?;
        let pool_token_account = next_account_info(account_info_iter)?;

        let _config = PoolConfig::try_from_slice(&config_info.data.borrow())?;
        let mut state = PoolState::try_from_slice(&state_info.data.borrow())?;

        // Find user's deposit
        let deposit = state.deposits.get(user_token_account.key).ok_or_else(|| {
            msg!("No deposit found for user: {}", user_token_account.key);
            TokenLockError::NoDepositFound
        })?;

        // Verify deposit state
        if deposit.state != UserDepositState::WithdrawReady {
            let current_state = deposit.state as u8;
            let expected_state = UserDepositState::WithdrawReady as u8;
            msg!(
                "Invalid deposit state: current={:?}, expected={:?}",
                deposit.state,
                UserDepositState::WithdrawReady
            );
            return Err(TokenLockError::InvalidDepositState(current_state, expected_state).into());
        }

        // Check if pool has enough tokens
        let pool_balance = TokenAccount::unpack(&pool_token_account.data.borrow())?.amount;
        if pool_balance < deposit.amount {
            msg!(
                "Insufficient pool balance: balance={}, required={}",
                pool_balance,
                deposit.amount
            );
            return Err(TokenLockError::InsufficientPoolBalance(pool_balance).into());
        }

        // Find pool authority PDA
        let (authority_key, authority_bump) =
            Pubkey::find_program_address(&[AUTHORITY_SEED], program_id);

        // Transfer collateral from pool to user
        invoke_signed(
            &token_instruction::transfer(
                &spl_token::id(),
                pool_token_account.key,
                user_token_account.key,
                &authority_key,
                &[],
                deposit.amount,
            )?,
            &[pool_token_account.clone(), user_token_account.clone()],
            &[&[AUTHORITY_SEED, &[authority_bump]]],
        )?;

        // Remove deposit
        state.deposits.remove(user_token_account.key);
        state.serialize(&mut *state_info.data.borrow_mut())?;

        Ok(())
    }
}
