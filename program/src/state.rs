use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Clone, Copy)]
pub enum UserDepositState {
    Deposited,
    WithdrawRequested,
    WithdrawReady,
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

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PoolConfig {
    pub interest_mint: Pubkey,
    pub collateral_mint: Pubkey,
    pub base_interest_rate: u64,
    pub price_factor: u64,
    pub min_commission_rate: u64,
    pub max_commission_rate: u64,
    pub min_deposit_amount: u64,
    pub max_deposit_amount: u64,
    pub deposit_period: Vec<u64>,
}

impl PoolConfig {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8; // pubkey(32) * 2 + u64(8) * 2 + u64(8) * 2 + u64(8) * 2 + u64(8)
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PoolState {
    pub deposits: HashMap<Pubkey, UserDeposit>,
}

impl PoolState {
    pub const LEN: usize = 4; // vec_len(4)
}
