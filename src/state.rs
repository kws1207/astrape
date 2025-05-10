use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub enum UserDepositState {
    Deposited,
    WithdrawRequested,
    WithdrawReady,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserDeposit {
    pub amount: u64,
    pub deposit_slot: u64, // Slot when the deposit was made
    pub unlock_slot: u64,  // Slot number for unlock time
    pub interest_received: u64,
    pub state: UserDepositState,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PoolConfig {
    pub admin: Pubkey,           // Admin pubkey
    pub interest_mint: Pubkey,   // immutable
    pub collateral_mint: Pubkey, // immutable
    pub base_interest_rate: u64, // Basis points (e.g., 500 = 5%)
    pub price_factor: u64,       // Basis points for price ratio (e.g., 10000 = 1:1, 20000 = 2:1)
}

impl PoolConfig {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 8; // pubkey(32) * 3 + u64(8) * 2
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PoolState {
    pub config: PoolConfig,
    pub deposits: HashMap<Pubkey, UserDeposit>,
}

impl PoolState {
    pub const LEN: usize = PoolConfig::LEN + 4; // config + vec_len(4)
}
