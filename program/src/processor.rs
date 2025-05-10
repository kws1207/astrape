use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use solana_sdk::program_pack::Pack;
use spl_token::{instruction as token_instruction, state::Account as TokenAccount};
use std::mem::size_of;

use crate::{
    errors::TokenLockError,
    instructions::TokenLockInstruction,
    state::{PoolConfig, PoolState, UserDeposit, UserDepositState},
};

// PDA seeds
const CONFIG_SEED: &[u8] = b"pool_config";
const INTEREST_POOL_SEED: &[u8] = b"interest_pool";
const COLLATERAL_POOL_SEED: &[u8] = b"collateral_pool";

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
        let instruction = TokenLockInstruction::try_from_slice(instruction_data)?;

        match instruction {
            TokenLockInstruction::AdminCreateConfig {
                interest_mint,
                collateral_mint,
                base_interest_rate,
                price_factor,
            } => Self::process_initialize(
                program_id,
                accounts,
                interest_mint,
                collateral_mint,
                base_interest_rate,
                price_factor,
            ),
            TokenLockInstruction::AdminUpdateConfig {
                param,
                base_interest_rate,
                price_factor,
            } => Self::process_update_config(
                program_id,
                accounts,
                param,
                base_interest_rate,
                price_factor,
            ),
            TokenLockInstruction::DepositCollateral { unlock_slot } => {
                Self::process_deposit_collateral(program_id, accounts, unlock_slot)
            }
            TokenLockInstruction::AdminWithdrawCollateralForInvestment => {
                Self::process_admin_withdraw_collateral_for_investment(program_id, accounts)
            }
            TokenLockInstruction::RequestWithdrawal => {
                Self::process_request_withdrawal(program_id, accounts)
            }
            TokenLockInstruction::AdminUpdateDepositStates => {
                Self::process_admin_update_deposit_states(program_id, accounts)
            }
            TokenLockInstruction::AdminPrepareWithdrawal { user_pubkey } => {
                Self::process_admin_prepare_withdrawal(program_id, accounts, user_pubkey)
            }
            TokenLockInstruction::WithdrawCollateral => {
                Self::process_withdraw_collateral(program_id, accounts)
            }
            TokenLockInstruction::AdminDepositInterest { amount } => {
                Self::process_admin_deposit_interest(program_id, accounts, amount)
            }
            TokenLockInstruction::AdminWithdrawInterest { amount } => {
                Self::process_admin_withdraw_interest(program_id, accounts, amount)
            }
        }
    }

    fn process_initialize(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        interest_mint: Pubkey,
        collateral_mint: Pubkey,
        base_interest_rate: u64,
        price_factor: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let interest_pool_account = next_account_info(account_info_iter)?;
        let collateral_pool_account = next_account_info(account_info_iter)?;

        // Verify admin
        if !admin_info.is_signer || config_feature::admin::id() != *admin_info.key {
            return Err(TokenLockError::InvalidAdmin.into());
        }

        // Verify system program
        if *system_program_info.key != solana_program::system_program::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Verify config PDA
        let (pda, bump_seed) = Pubkey::find_program_address(&[CONFIG_SEED], program_id);
        if pda != *config_info.key {
            return Err(TokenLockError::InvalidPoolAccount.into());
        }

        // Check if account is already initialized
        if config_info.owner != system_program_info.key {
            return Err(TokenLockError::InvalidPoolAccount.into());
        }

        // Verify pool accounts are PDAs
        let (expected_interest_pool, expected_collateral_pool) =
            Self::find_pool_accounts(program_id, &interest_mint, &collateral_mint);
        if *interest_pool_account.key != expected_interest_pool {
            return Err(TokenLockError::InvalidPoolAccount.into());
        }
        if *collateral_pool_account.key != expected_collateral_pool {
            return Err(TokenLockError::InvalidPoolAccount.into());
        }

        // Calculate rent and allocate space
        let pda_signer_seeds: &[&[_]] = &[CONFIG_SEED, &[bump_seed]];
        let rent = Rent::get()?;
        let data_size = size_of::<PoolConfig>();
        let required_lamports = rent
            .minimum_balance(data_size)
            .max(1)
            .saturating_sub(config_info.lamports());

        // Transfer lamports if needed
        if required_lamports > 0 {
            invoke(
                &system_instruction::transfer(admin_info.key, config_info.key, required_lamports),
                &[
                    admin_info.clone(),
                    config_info.clone(),
                    system_program_info.clone(),
                ],
            )?;
        }

        // Allocate space
        invoke_signed(
            &system_instruction::allocate(config_info.key, data_size as u64),
            &[config_info.clone(), system_program_info.clone()],
            &[pda_signer_seeds],
        )?;

        // Assign to program
        invoke_signed(
            &system_instruction::assign(config_info.key, program_id),
            &[config_info.clone(), system_program_info.clone()],
            &[pda_signer_seeds],
        )?;

        // Initialize config with provided values
        let config = PoolConfig {
            admin: *admin_info.key,
            interest_mint,
            collateral_mint,
            base_interest_rate,
            price_factor,
        };
        config.serialize(&mut *config_info.data.borrow_mut())?;

        Ok(())
    }

    fn process_update_config(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        param: u8,
        base_interest_rate: Option<u64>,
        price_factor: Option<u64>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        let config_info = next_account_info(account_info_iter)?;

        // Verify admin
        if !admin_info.is_signer || config_feature::admin::id() != *admin_info.key {
            return Err(TokenLockError::InvalidAdmin.into());
        }

        // Verify config PDA
        let (pda, _) = Pubkey::find_program_address(&[CONFIG_SEED], program_id);
        if pda != *config_info.key || config_info.owner != program_id {
            return Err(TokenLockError::InvalidPoolAccount.into());
        }

        // Update config based on parameter
        let mut config = PoolConfig::try_from_slice(&config_info.data.borrow())?;
        match param {
            0 => {
                if let Some(rate) = base_interest_rate {
                    config.base_interest_rate = rate;
                }
            }
            1 => {
                if let Some(factor) = price_factor {
                    if factor == 0 {
                        return Err(TokenLockError::InvalidInput.into());
                    }
                    config.price_factor = factor;
                }
            }
            _ => return Err(TokenLockError::InvalidInput.into()),
        }

        config.serialize(&mut *config_info.data.borrow_mut())?;
        Ok(())
    }

    fn find_pool_accounts(
        program_id: &Pubkey,
        interest_mint: &Pubkey,
        collateral_mint: &Pubkey,
    ) -> (Pubkey, Pubkey) {
        let (interest_pool, _) =
            Pubkey::find_program_address(&[INTEREST_POOL_SEED, interest_mint.as_ref()], program_id);
        let (collateral_pool, _) = Pubkey::find_program_address(
            &[COLLATERAL_POOL_SEED, collateral_mint.as_ref()],
            program_id,
        );
        (interest_pool, collateral_pool)
    }

    fn process_deposit_collateral(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        unlock_slot: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let pool_state_account = next_account_info(account_info_iter)?;
        let user_token_account = next_account_info(account_info_iter)?;
        let pool_token_account = next_account_info(account_info_iter)?;
        let user_interest_account = next_account_info(account_info_iter)?;
        let interest_pool_account = next_account_info(account_info_iter)?;

        let mut pool_state = PoolState::try_from_slice(&pool_state_account.data.borrow())?;
        let clock = Clock::get()?;

        // Verify pool accounts are PDAs
        let (expected_interest_pool, expected_collateral_pool) = Self::find_pool_accounts(
            program_id,
            &pool_state.config.interest_mint,
            &pool_state.config.collateral_mint,
        );
        if *interest_pool_account.key != expected_interest_pool {
            return Err(TokenLockError::InvalidPoolAccount.into());
        }
        if *pool_token_account.key != expected_collateral_pool {
            return Err(TokenLockError::InvalidPoolAccount.into());
        }

        // Verify unlock slot is in the future
        if unlock_slot <= clock.slot {
            return Err(TokenLockError::InvalidLockPeriod.into());
        }

        // Get user's collateral balance
        let amount = TokenAccount::unpack(&user_token_account.data.borrow())?.amount;
        if amount == 0 {
            return Err(TokenLockError::InsufficientBalance.into());
        }

        // Calculate interest based on slot duration and price factor
        let slot_duration = unlock_slot - clock.slot;
        let interest_multiplier = (slot_duration as u128)
            .checked_mul(pool_state.config.base_interest_rate as u128)
            .unwrap()
            .checked_div(365 * 24 * 60 * 60 * 2) // Convert to annual rate (assuming 2 slots per second)
            .unwrap();

        // Calculate interest amount with price factor
        let interest_amount = (amount as u128)
            .checked_mul(interest_multiplier)
            .unwrap()
            .checked_mul(pool_state.config.price_factor as u128)
            .unwrap()
            .checked_div(10000)
            .unwrap()
            .checked_div(10000)
            .unwrap() as u64;

        // Check if pool has enough interest tokens
        let interest_pool_balance =
            TokenAccount::unpack(&interest_pool_account.data.borrow())?.amount;
        if interest_pool_balance < interest_amount {
            return Err(TokenLockError::InsufficientPoolBalance.into());
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

        // Transfer interest to user
        invoke(
            &token_instruction::transfer(
                &spl_token::id(),
                interest_pool_account.key,
                user_interest_account.key,
                &pool_state_account.key,
                &[],
                interest_amount,
            )?,
            &[
                interest_pool_account.clone(),
                user_interest_account.clone(),
                pool_state_account.clone(),
            ],
        )?;

        // Add user deposit
        let user_deposit = UserDeposit {
            amount,
            deposit_slot: clock.slot,
            unlock_slot,
            interest_received: interest_amount,
            state: UserDepositState::Deposited,
        };
        pool_state
            .deposits
            .insert(*user_token_account.key, user_deposit);

        pool_state.serialize(&mut *pool_state_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_admin_withdraw_collateral_for_investment(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let pool_state_account = next_account_info(account_info_iter)?;
        let admin_token_account = next_account_info(account_info_iter)?;
        let pool_token_account = next_account_info(account_info_iter)?;

        let mut pool_state = PoolState::try_from_slice(&pool_state_account.data.borrow())?;

        // Verify admin
        if *admin_token_account.key != pool_state.config.admin {
            return Err(TokenLockError::InvalidAdmin.into());
        }

        // Get pool's collateral balance
        let amount = TokenAccount::unpack(&pool_token_account.data.borrow())?.amount;
        if amount == 0 {
            return Err(TokenLockError::InsufficientPoolBalance.into());
        }

        // Transfer collateral to admin
        invoke(
            &token_instruction::transfer(
                &spl_token::id(),
                pool_token_account.key,
                admin_token_account.key,
                &pool_state_account.key,
                &[],
                amount,
            )?,
            &[
                pool_token_account.clone(),
                admin_token_account.clone(),
                pool_state_account.clone(),
            ],
        )?;

        Ok(())
    }

    fn process_request_withdrawal(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let pool_state_account = next_account_info(account_info_iter)?;
        let user_interest_account = next_account_info(account_info_iter)?;
        let interest_pool_account = next_account_info(account_info_iter)?;

        let mut pool_state = PoolState::try_from_slice(&pool_state_account.data.borrow())?;
        let clock = Clock::get()?;

        // Find user's deposit
        let deposit = pool_state
            .deposits
            .get(user_interest_account.key)
            .ok_or(TokenLockError::NoDepositFound)?;

        // Verify deposit state
        if deposit.state != UserDepositState::Deposited {
            return Err(TokenLockError::InvalidDepositState.into());
        }

        // Calculate remaining interest to be returned based on actual lock duration
        let actual_lock_duration = clock.slot - deposit.deposit_slot;
        let total_lock_duration = deposit.unlock_slot - deposit.deposit_slot;
        let interest_to_return = (deposit.interest_received as u128)
            .checked_mul(actual_lock_duration as u128)
            .unwrap()
            .checked_div(total_lock_duration as u128)
            .unwrap() as u64;

        // Check if user has enough interest tokens to return
        let user_interest_balance =
            TokenAccount::unpack(&user_interest_account.data.borrow())?.amount;
        if user_interest_balance < interest_to_return {
            return Err(TokenLockError::InsufficientInterestBalance.into());
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
        };
        pool_state
            .deposits
            .insert(*user_interest_account.key, updated_deposit);

        pool_state.serialize(&mut *pool_state_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_admin_update_deposit_states(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let pool_state_account = next_account_info(account_info_iter)?;
        let clock_sysvar = next_account_info(account_info_iter)?;

        let mut pool_state = PoolState::try_from_slice(&pool_state_account.data.borrow())?;
        let clock = Clock::from_account_info(clock_sysvar)?;

        // Update states for deposits that have reached their unlock slot
        for (_, deposit) in pool_state.deposits.iter_mut() {
            if deposit.state == UserDepositState::Deposited && deposit.unlock_slot <= clock.slot {
                deposit.state = UserDepositState::WithdrawRequested;
            }
        }

        pool_state.serialize(&mut *pool_state_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_admin_prepare_withdrawal(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        user_pubkey: Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let pool_state_account = next_account_info(account_info_iter)?;
        let admin_token_account = next_account_info(account_info_iter)?;
        let user_token_account = next_account_info(account_info_iter)?;
        let pool_token_account = next_account_info(account_info_iter)?;

        let mut pool_state = PoolState::try_from_slice(&pool_state_account.data.borrow())?;

        // Verify admin
        if *admin_token_account.key != pool_state.config.admin {
            return Err(TokenLockError::InvalidAdmin.into());
        }

        // Find user's deposit
        let deposit = pool_state
            .deposits
            .get(&user_pubkey)
            .ok_or(TokenLockError::NoDepositFound)?;

        // Verify deposit state
        if deposit.state != UserDepositState::WithdrawRequested {
            return Err(TokenLockError::InvalidDepositState.into());
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
        };
        pool_state.deposits.insert(user_pubkey, updated_deposit);

        pool_state.serialize(&mut *pool_state_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_withdraw_collateral(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let pool_state_account = next_account_info(account_info_iter)?;
        let user_token_account = next_account_info(account_info_iter)?;
        let pool_token_account = next_account_info(account_info_iter)?;

        let mut pool_state = PoolState::try_from_slice(&pool_state_account.data.borrow())?;

        // Find user's deposit
        let deposit = pool_state
            .deposits
            .get(user_token_account.key)
            .ok_or(TokenLockError::NoDepositFound)?;

        // Verify deposit state
        if deposit.state != UserDepositState::WithdrawReady {
            return Err(TokenLockError::InvalidDepositState.into());
        }

        // Transfer collateral from pool to user
        invoke(
            &token_instruction::transfer(
                &spl_token::id(),
                pool_token_account.key,
                user_token_account.key,
                &pool_state_account.key,
                &[],
                deposit.amount,
            )?,
            &[
                pool_token_account.clone(),
                user_token_account.clone(),
                pool_state_account.clone(),
            ],
        )?;

        // Remove deposit
        pool_state.deposits.remove(user_token_account.key);
        pool_state.serialize(&mut *pool_state_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_admin_deposit_interest(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let pool_state_account = next_account_info(account_info_iter)?;
        let admin_interest_account = next_account_info(account_info_iter)?;
        let interest_pool_account = next_account_info(account_info_iter)?;

        let pool_state = PoolState::try_from_slice(&pool_state_account.data.borrow())?;

        // Verify admin
        if *admin_interest_account.key != config_feature::admin::id() {
            return Err(TokenLockError::InvalidAdmin.into());
        }

        // Verify pool account is PDA
        let (expected_interest_pool, _) = Self::find_pool_accounts(
            program_id,
            &pool_state.config.interest_mint,
            &pool_state.config.collateral_mint,
        );
        if *interest_pool_account.key != expected_interest_pool {
            return Err(TokenLockError::InvalidPoolAccount.into());
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
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let pool_state_account = next_account_info(account_info_iter)?;
        let admin_interest_account = next_account_info(account_info_iter)?;
        let interest_pool_account = next_account_info(account_info_iter)?;

        let pool_state = PoolState::try_from_slice(&pool_state_account.data.borrow())?;

        // Verify admin
        if *admin_interest_account.key != config_feature::admin::id() {
            return Err(TokenLockError::InvalidAdmin.into());
        }

        // Verify pool account is PDA
        let (expected_interest_pool, _) = Self::find_pool_accounts(
            program_id,
            &pool_state.config.interest_mint,
            &pool_state.config.collateral_mint,
        );
        if *interest_pool_account.key != expected_interest_pool {
            return Err(TokenLockError::InvalidPoolAccount.into());
        }

        // Transfer interest from pool to admin
        invoke(
            &token_instruction::transfer(
                &spl_token::id(),
                interest_pool_account.key,
                admin_interest_account.key,
                &pool_state_account.key,
                &[],
                amount,
            )?,
            &[
                interest_pool_account.clone(),
                admin_interest_account.clone(),
                pool_state_account.clone(),
            ],
        )?;

        Ok(())
    }
}
