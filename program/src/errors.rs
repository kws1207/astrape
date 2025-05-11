use solana_program::{msg, program_error::ProgramError};
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, PartialEq)]
pub enum TokenLockError {
    // Basic instruction errors
    #[error("Invalid instruction code: {0}")]
    InvalidInstruction(u8),

    // Authentication errors
    #[error("Invalid admin: expected {0}")]
    InvalidAdmin(u8),

    #[error("Signer required")]
    SignerRequired,

    // Account validation errors
    #[error("Invalid pool account: {0}")]
    InvalidPoolAccount(u8),

    #[error("Account not owned by program")]
    InvalidAccountOwner,

    #[error("Invalid mint for pool")]
    InvalidMint,

    #[error("Expected PDA account does not match: {0}")]
    InvalidPDA(u8),

    #[error("Account already initialized")]
    AccountAlreadyInitialized,

    #[error("Account not initialized")]
    AccountNotInitialized,

    // Operation errors
    #[error("Invalid lock period: {0}")]
    InvalidLockPeriod(u64),

    #[error("Invalid configuration parameter: {0}")]
    InvalidConfigParam(u8),

    #[error("Amount below minimum: {0}")]
    AmountBelowMinimum(u64),

    #[error("Amount exceeds maximum: {0}")]
    AmountExceedsMaximum(u64),

    #[error("Value out of range: {0}")]
    ValueOutOfRange(u64),

    #[error("Insufficient balance: {0}")]
    InsufficientBalance(u64),

    #[error("Insufficient pool balance: {0}")]
    InsufficientPoolBalance(u64),

    #[error("Insufficient interest balance: {0}")]
    InsufficientInterestBalance(u64),

    // State errors
    #[error("No deposit found for user")]
    NoDepositFound,

    #[error("Invalid deposit state, current: {0}, expected: {1}")]
    InvalidDepositState(u8, u8),

    #[error("Operation not allowed at this time: {0}")]
    OperationNotAllowed(u8),

    #[error("Lock period not yet expired")]
    LockPeriodNotExpired,

    // Mathematical errors
    #[error("Arithmetic overflow")]
    ArithmeticOverflow,

    #[error("Division by zero")]
    DivisionByZero,

    // Generic errors
    #[error("Invalid input")]
    InvalidInput,

    #[error("Unexpected error")]
    Unexpected,
}

// Error code base for token lock errors
const TOKEN_LOCK_ERROR_CODE_BASE: u32 = 1000;

// Mapping from TokenLockError to u32 error codes
impl From<TokenLockError> for ProgramError {
    fn from(e: TokenLockError) -> Self {
        // Log the detailed error for on-chain visibility
        msg!("Token Lock Error: {}", e);

        // Add a specific offset to generate custom error codes
        let error_code = match e {
            TokenLockError::InvalidInstruction(_) => 0,
            TokenLockError::InvalidAdmin(_) => 1,
            TokenLockError::SignerRequired => 2,
            TokenLockError::InvalidPoolAccount(_) => 3,
            TokenLockError::InvalidAccountOwner => 4,
            TokenLockError::InvalidMint => 5,
            TokenLockError::InvalidPDA(_) => 6,
            TokenLockError::AccountAlreadyInitialized => 7,
            TokenLockError::AccountNotInitialized => 8,
            TokenLockError::InvalidLockPeriod(_) => 9,
            TokenLockError::InvalidConfigParam(_) => 10,
            TokenLockError::AmountBelowMinimum(_) => 11,
            TokenLockError::AmountExceedsMaximum(_) => 12,
            TokenLockError::ValueOutOfRange(_) => 13,
            TokenLockError::InsufficientBalance(_) => 14,
            TokenLockError::InsufficientPoolBalance(_) => 15,
            TokenLockError::InsufficientInterestBalance(_) => 16,
            TokenLockError::NoDepositFound => 17,
            TokenLockError::InvalidDepositState(_, _) => 18,
            TokenLockError::OperationNotAllowed(_) => 19,
            TokenLockError::LockPeriodNotExpired => 20,
            TokenLockError::ArithmeticOverflow => 21,
            TokenLockError::DivisionByZero => 22,
            TokenLockError::InvalidInput => 23,
            TokenLockError::Unexpected => 24,
        };

        ProgramError::Custom(TOKEN_LOCK_ERROR_CODE_BASE + error_code)
    }
}

// Helper methods for better error context
impl TokenLockError {
    pub fn log_and_return<T>(self) -> Result<T, ProgramError> {
        msg!("Token Lock Error: {:?}", self);
        Err(self.into())
    }

    pub fn invalid_account_owner(owner_pubkey: &str) -> Self {
        msg!("Invalid account owner: {}", owner_pubkey);
        Self::InvalidAccountOwner
    }

    pub fn insufficient_balance(available: u64, required: u64) -> Self {
        msg!(
            "Insufficient balance: available={}, required={}",
            available,
            required
        );
        Self::InsufficientBalance(available)
    }

    pub fn invalid_mint(expected: &str, actual: &str) -> Self {
        msg!("Invalid mint: expected={}, actual={}", expected, actual);
        Self::InvalidMint
    }
}

// Extension trait for Result to make error handling cleaner
pub trait TokenLockResult<T> {
    fn log_error(self) -> Result<T, ProgramError>;
    fn with_context(self, error_context: &str) -> Result<T, ProgramError>;
}

impl<T> TokenLockResult<T> for Result<T, TokenLockError> {
    fn log_error(self) -> Result<T, ProgramError> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => {
                msg!("Token Lock Error: {:?}", error);
                Err(error.into())
            }
        }
    }

    fn with_context(self, error_context: &str) -> Result<T, ProgramError> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => {
                msg!("Error Context: {}", error_context);
                msg!("Token Lock Error: {:?}", error);
                Err(error.into())
            }
        }
    }
}
