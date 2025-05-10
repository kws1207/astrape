use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum TokenLockError {
    #[error("Invalid instruction")]
    InvalidInstruction,

    #[error("Invalid admin")]
    InvalidAdmin,

    #[error("Invalid pool account")]
    InvalidPoolAccount,

    #[error("Invalid lock period")]
    InvalidLockPeriod,

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Insufficient pool balance")]
    InsufficientPoolBalance,

    #[error("Insufficient interest balance")]
    InsufficientInterestBalance,

    #[error("No deposit found")]
    NoDepositFound,

    #[error("Invalid deposit state")]
    InvalidDepositState,

    #[error("Invalid input")]
    InvalidInput,
}

impl From<TokenLockError> for ProgramError {
    fn from(e: TokenLockError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
