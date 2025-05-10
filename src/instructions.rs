use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum TokenLockInstruction {
    /// Initialize the pool configuration
    ///
    /// Accounts expected:
    /// 0. `[signer]` Admin account
    /// 1. `[writable]` Config PDA account
    /// 2. `[]` System program
    /// 3. `[]` Interest pool PDA account
    /// 4. `[]` Collateral pool PDA account
    AdminCreateConfig {
        interest_mint: Pubkey,
        collateral_mint: Pubkey,
        base_interest_rate: u64,
        price_factor: u64,
    },

    /// Update pool configuration parameters
    ///
    /// Accounts expected:
    /// 0. `[signer]` Admin account
    /// 1. `[writable]` Config PDA account
    AdminUpdateConfig {
        param: u8,
        base_interest_rate: Option<u64>,
        price_factor: Option<u64>,
    },

    /// Deposit collateral tokens and receive interest tokens
    ///
    /// Accounts expected:
    /// 0. `[writable]` Pool state account
    /// 1. `[writable]` User's collateral token account
    /// 2. `[writable]` Pool's collateral token account
    /// 3. `[writable]` User's interest token account
    /// 4. `[writable]` Pool's interest token account
    DepositCollateral { unlock_slot: u64 },

    /// Admin withdraws collateral for investment
    ///
    /// Accounts expected:
    /// 0. `[writable]` Pool state account
    /// 1. `[writable]` Admin's collateral token account
    /// 2. `[writable]` Pool's collateral token account
    AdminWithdrawCollateralForInvestment,

    /// Request withdrawal of collateral
    ///
    /// Accounts expected:
    /// 0. `[writable]` Pool state account
    /// 1. `[writable]` User's interest token account
    /// 2. `[writable]` Pool's interest token account
    RequestWithdrawal,

    /// Admin updates deposit states based on current slot
    ///
    /// Accounts expected:
    /// 0. `[writable]` Pool state account
    AdminUpdateDepositStates,

    /// Admin prepares withdrawal by depositing collateral
    ///
    /// Accounts expected:
    /// 0. `[writable]` Pool state account
    /// 1. `[writable]` Admin's collateral token account
    /// 2. `[writable]` User's collateral token account
    /// 3. `[writable]` Pool's collateral token account
    AdminPrepareWithdrawal { user_pubkey: Pubkey },

    /// Withdraw collateral after admin preparation
    ///
    /// Accounts expected:
    /// 0. `[writable]` Pool state account
    /// 1. `[writable]` User's collateral token account
    /// 2. `[writable]` Pool's collateral token account
    WithdrawCollateral,

    /// Admin deposits interest tokens to the pool
    ///
    /// Accounts expected:
    /// 0. `[writable]` Pool state account
    /// 1. `[writable]` Admin's interest token account
    /// 2. `[writable]` Pool's interest token account
    AdminDepositInterest { amount: u64 },

    /// Admin withdraws interest tokens from the pool
    ///
    /// Accounts expected:
    /// 0. `[writable]` Pool state account
    /// 1. `[writable]` Admin's interest token account
    /// 2. `[writable]` Pool's interest token account
    AdminWithdrawInterest { amount: u64 },
}

impl TokenLockInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let instruction = Self::try_from_slice(input)?;
        Ok(instruction)
    }
    pub fn pack(&self) -> Result<Vec<u8>, ProgramError> {
        let mut buffer = vec![];
        match self {
            Self::AdminCreateConfig {
                interest_mint,
                collateral_mint,
                base_interest_rate,
                price_factor,
            } => {
                buffer.push(0);
                buffer.extend_from_slice(&interest_mint.to_bytes());
                buffer.extend_from_slice(&collateral_mint.to_bytes());
                buffer.extend_from_slice(&base_interest_rate.to_le_bytes());
                buffer.extend_from_slice(&price_factor.to_le_bytes());
            }
            Self::AdminUpdateConfig {
                param,
                base_interest_rate,
                price_factor,
            } => {
                buffer.push(1);
                buffer.push(*param);
                if let Some(rate) = base_interest_rate {
                    buffer.push(1);
                    buffer.extend_from_slice(&rate.to_le_bytes());
                } else {
                    buffer.push(0);
                }
                if let Some(factor) = price_factor {
                    buffer.push(1);
                    buffer.extend_from_slice(&factor.to_le_bytes());
                } else {
                    buffer.push(0);
                }
            }
            Self::DepositCollateral { unlock_slot } => {
                buffer.push(2);
                buffer.extend_from_slice(&unlock_slot.to_le_bytes());
            }
            Self::AdminWithdrawCollateralForInvestment => {
                buffer.push(3);
            }
            Self::RequestWithdrawal => {
                buffer.push(4);
            }
            Self::AdminUpdateDepositStates => {
                buffer.push(5);
            }
            Self::AdminPrepareWithdrawal { user_pubkey } => {
                buffer.push(6);
                buffer.extend_from_slice(&user_pubkey.to_bytes());
            }
            Self::WithdrawCollateral => {
                buffer.push(7);
            }
            Self::AdminDepositInterest { amount } => {
                buffer.push(8);
                buffer.extend_from_slice(&amount.to_le_bytes());
            }
            Self::AdminWithdrawInterest { amount } => {
                buffer.push(9);
                buffer.extend_from_slice(&amount.to_le_bytes());
            }
        }
        Ok(buffer)
    }
}
