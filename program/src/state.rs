use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Clone, Copy)]
pub enum UserDepositState {
    Deposited,
    WithdrawRequested,
    WithdrawReady,
    WithdrawCompleted,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UserDeposit {
    pub amount: u64,
    pub deposit_slot: u64, // Slot when the deposit was made
    pub unlock_slot: u64,  // Slot number for unlock time
    pub interest_received: u64,
    pub state: UserDepositState,

    pub commission_rate: u64,
}

impl UserDeposit {
    pub const LEN: usize = 8 + 8 + 8 + 8 + size_of::<UserDepositState>() + 8;
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AstrapeConfig {
    pub interest_mint: Pubkey,
    pub collateral_mint: Pubkey,
    pub base_interest_rate: u64, // Fixed decimal at the first decimal place
    pub price_factor: u64, // collateral token amount * price_factor = interest token amount for the same value (decimal considered)

    pub min_commission_rate: u64, // Fixed decimal at the first decimal place
    pub max_commission_rate: u64, // Fixed decimal at the first decimal place
    pub min_deposit_amount: u64,
    pub max_deposit_amount: u64,
    pub deposit_periods: Vec<u64>,
}

impl AstrapeConfig {
    pub const LEN: usize = 32 * 2 + 8 * 6 + 8 * 3 + 4; // size_of::<Vec<u64>>(); // 160
}
