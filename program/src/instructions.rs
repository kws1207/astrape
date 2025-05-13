use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum TokenLockInstruction {
    /// Initialize the pool with configuration and create necessary PDAs
    ///
    /// Accounts expected:
    /// 0. `[signer]` Admin account
    /// 1. `[writable]` Config PDA account
    /// 2. `[writable]` Authority PDA account
    /// 3. `[writable]` Interest pool ATA account
    /// 4. `[writable]` Collateral pool ATA account
    /// 5. `[writable]` Withdrawal pool account
    /// 6. `[]` Interest mint account
    /// 7. `[]` Collateral mint account
    /// 8. `[]` System program
    /// 9. `[]` Token program
    /// 10. `[]` Associated Token Account program
    /// 11. `[]` Rent sysvar
    Initialize {
        interest_mint: Pubkey,
        collateral_mint: Pubkey,
        base_interest_rate: u64,
        price_factor: u64,
        min_commission_rate: u64,
        max_commission_rate: u64,
        min_deposit_amount: u64,
        max_deposit_amount: u64,
        deposit_periods: Vec<u64>,
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
        min_commission_rate: Option<u64>,
        max_commission_rate: Option<u64>,
        min_deposit_amount: Option<u64>,
        max_deposit_amount: Option<u64>,
        deposit_periods: Option<Vec<u64>>,
    },

    /// Admin withdraws collateral for investment
    ///
    /// Accounts expected:
    /// 0. `[signer]` Admin account
    /// 1. `[]` Config PDA account
    /// 2. `[]` Authority PDA account
    /// 3. `[writable]` Admin's collateral token account
    /// 4. `[writable]` Pool's collateral token account
    /// 5. `[]` System program
    /// 6. `[]` Token program
    /// 7. `[]` Associated Token Account program
    AdminWithdrawCollateralForInvestment,

    /// Admin prepares withdrawal by depositing collateral
    ///
    /// Accounts expected:
    /// 0. `[signer]` Admin account
    /// 1. `[]` Config PDA account
    /// 2. `[writable]` Admin's collateral token account
    /// 3. `[writable]` Withdrawal pool account
    /// 4. `[]` User account
    /// 5. `[writable]` User deposit account
    /// 6. `[]` Token program
    AdminPrepareWithdrawal,

    /// Admin deposits interest tokens to the pool
    ///
    /// Accounts expected:
    /// 0. `[signer]` Admin account
    /// 1. `[]` Config PDA account
    /// 2. `[]` Authority PDA account
    /// 3. `[writable]` Admin's interest token account
    /// 4. `[writable]` Pool's interest token account
    /// 5. `[]` System program
    /// 6. `[]` Token program
    /// 7. `[]` Associated Token Account program
    AdminDepositInterest { amount: u64 },

    /// Admin withdraws interest tokens from the pool
    ///
    /// Accounts expected:
    /// 0. `[signer]` Admin account
    /// 1. `[]` Config PDA account
    /// 2. `[]` Authority PDA account
    /// 3. `[writable]` Admin's interest token account
    /// 4. `[writable]` Pool's interest token account
    /// 5. `[]` System program
    /// 6. `[]` Token program
    /// 7. `[]` Associated Token Account program
    AdminWithdrawInterest { amount: u64 },

    /// Deposit collateral tokens into the pool
    ///
    /// Accounts expected:
    /// 0. `[signer]` User account
    /// 1. `[]` Config PDA account
    /// 2. `[]` Authority PDA account
    /// 3. `[writable]` User's collateral token account
    /// 4. `[writable]` User's deposit account
    /// 5. `[writable]` Pool's collateral token account
    /// 6. `[writable]` User's interest token account
    /// 7. `[writable]` Pool's interest token account
    /// 8. `[]` System program
    /// 9. `[]` Token program
    DepositCollateral {
        amount: u64,
        deposit_period: u64,
        commission_rate: u64,
    },

    /// Request early withdrawal of collateral (before unlock time)
    ///
    /// Accounts expected:
    /// 0. `[signer]` User account
    /// 1. `[]` Config PDA account
    /// 2. `[]` Authority PDA account
    /// 3. `[writable]` User's deposit account
    /// 4. `[writable]` User's interest token account
    /// 5. `[writable]` Pool's interest token account
    /// 6. `[]` Token program
    RequestWithdrawalEarly,

    /// Request withdrawal of collateral (after unlock time)
    ///
    /// Accounts expected:
    /// 0. `[signer]` User account
    /// 1. `[writable]` User's deposit account
    RequestWithdrawal,

    /// Withdraw collateral after admin preparation
    ///
    /// Accounts expected:
    /// 0. `[signer]` User account
    /// 1. `[]` Config PDA account
    /// 2. `[]` Authority PDA account
    /// 3. `[writable]` User's deposit account
    /// 4. `[writable]` User's collateral token account
    /// 5. `[writable]` Withdrawal pool account
    /// 6. `[]` Token program
    WithdrawCollateral,
}

impl TokenLockInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let instruction = Self::try_from_slice(input)?;
        Ok(instruction)
    }
    pub fn pack(&self) -> Result<Vec<u8>, ProgramError> {
        let mut buffer = vec![];
        match self {
            Self::Initialize {
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
                buffer.push(0);
                buffer.extend_from_slice(&interest_mint.to_bytes());
                buffer.extend_from_slice(&collateral_mint.to_bytes());
                buffer.extend_from_slice(&base_interest_rate.to_le_bytes());
                buffer.extend_from_slice(&price_factor.to_le_bytes());
                buffer.extend_from_slice(&min_commission_rate.to_le_bytes());
                buffer.extend_from_slice(&max_commission_rate.to_le_bytes());
                buffer.extend_from_slice(&min_deposit_amount.to_le_bytes());
                buffer.extend_from_slice(&max_deposit_amount.to_le_bytes());
                buffer.extend_from_slice(
                    &deposit_periods
                        .iter()
                        .flat_map(|&x| x.to_le_bytes())
                        .collect::<Vec<u8>>(),
                );
            }
            Self::AdminUpdateConfig {
                param,
                base_interest_rate,
                price_factor,
                min_commission_rate,
                max_commission_rate,
                min_deposit_amount,
                max_deposit_amount,
                deposit_periods,
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
                if let Some(min_commission_rate) = min_commission_rate {
                    buffer.push(1);
                    buffer.extend_from_slice(&min_commission_rate.to_le_bytes());
                } else {
                    buffer.push(0);
                }
                if let Some(max_commission_rate) = max_commission_rate {
                    buffer.push(1);
                    buffer.extend_from_slice(&max_commission_rate.to_le_bytes());
                } else {
                    buffer.push(0);
                }
                if let Some(min_deposit_amount) = min_deposit_amount {
                    buffer.push(1);
                    buffer.extend_from_slice(&min_deposit_amount.to_le_bytes());
                } else {
                    buffer.push(0);
                }
                if let Some(max_deposit_amount) = max_deposit_amount {
                    buffer.push(1);
                    buffer.extend_from_slice(&max_deposit_amount.to_le_bytes());
                } else {
                    buffer.push(0);
                }
                if let Some(deposit_periods) = deposit_periods {
                    buffer.push(1);
                    buffer.extend_from_slice(
                        &deposit_periods
                            .iter()
                            .flat_map(|&x| x.to_le_bytes())
                            .collect::<Vec<u8>>(),
                    );
                } else {
                    buffer.push(0);
                }
            }
            Self::AdminWithdrawCollateralForInvestment => {
                buffer.push(2);
            }
            Self::AdminPrepareWithdrawal => {
                buffer.push(3);
            }
            Self::AdminDepositInterest { amount } => {
                buffer.push(4);
                buffer.extend_from_slice(&amount.to_le_bytes());
            }
            Self::AdminWithdrawInterest { amount } => {
                buffer.push(5);
                buffer.extend_from_slice(&amount.to_le_bytes());
            }
            Self::DepositCollateral {
                amount,
                deposit_period,
                commission_rate: comminsion_rate,
            } => {
                buffer.push(6);
                buffer.extend_from_slice(&amount.to_le_bytes());
                buffer.extend_from_slice(&deposit_period.to_le_bytes());
                buffer.extend_from_slice(&comminsion_rate.to_le_bytes());
            }
            Self::RequestWithdrawalEarly => {
                buffer.push(7);
            }
            Self::RequestWithdrawal => {
                buffer.push(8);
            }
            Self::WithdrawCollateral => {
                buffer.push(9);
            }
        }
        Ok(buffer)
    }
}
