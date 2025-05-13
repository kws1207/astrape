use anyhow::{anyhow, Result};
use astrape_admin_utils::PROGRAM_ID;
use borsh::BorshSerialize;
use breakout_contract::{
    instructions::TokenLockInstruction,
    processor::{CONFIG_SEED, SLOTS_PER_MONTH},
};
use clap::Parser;
use solana_client::rpc_client::RpcClient;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{read_keypair_file, Signer},
    transaction::Transaction,
};
use std::str::FromStr;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Keypair file for the admin account
    #[arg(short, long)]
    keypair: String,

    /// URL of the Solana cluster
    #[arg(short, long, default_value = "https://api.devnet.solana.com")]
    url: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Parse Solana program ID
    let program_id = Pubkey::from_str(PROGRAM_ID)?;

    // Initialize RPC client with the specified URL
    let rpc_client = RpcClient::new_with_commitment(&args.url, CommitmentConfig::confirmed());
    println!("Connected to Solana cluster at {}", args.url);

    // Load admin keypair
    let admin_keypair = read_keypair_file(&args.keypair)
        .map_err(|_| anyhow!("Failed to read keypair file: {}", args.keypair))?;
    println!("Admin pubkey: {}", admin_keypair.pubkey());
    // Find PDAs
    let (config_pda, _) = Pubkey::find_program_address(&[CONFIG_SEED], &program_id);
    println!("Config PDA: {}", config_pda);

    let new_config = TokenLockInstruction::AdminUpdateConfig {
        param: 0,
        base_interest_rate: Some(170),
        price_factor: Some(100_000 / 10_u64.pow(8 - 6)),
        min_commission_rate: Some(200),
        max_commission_rate: Some(500),
        min_deposit_amount: Some(10_000_000),
        max_deposit_amount: Some(1_000_000_000),
        deposit_periods: Some(vec![
            1 * SLOTS_PER_MONTH as u64,
            3 * SLOTS_PER_MONTH as u64,
            6 * SLOTS_PER_MONTH as u64,
        ]),
    };

    let instructions = {
        let mut instructions = vec![];
        for i in 0..=6 {
            let instruction = Instruction {
                program_id,
                accounts: vec![
                    AccountMeta::new(admin_keypair.pubkey(), true),
                    AccountMeta::new(config_pda, false),
                ],
                data: TokenLockInstruction::AdminUpdateConfig {
                    param: i,
                    base_interest_rate: Some(170),
                    price_factor: Some(100_000 / 10_u64.pow(8 - 6)),
                    min_commission_rate: Some(200),
                    max_commission_rate: Some(500),
                    min_deposit_amount: Some(10_000_000),
                    max_deposit_amount: Some(1_000_000_000),
                    deposit_periods: Some(vec![
                        1 * SLOTS_PER_MONTH as u64,
                        3 * SLOTS_PER_MONTH as u64,
                        6 * SLOTS_PER_MONTH as u64,
                    ]),
                }
                .try_to_vec()
                .unwrap(),
            };
            instructions.push(instruction);
        }
        instructions
    };

    let mut transaction = Transaction::new_with_payer(&instructions, Some(&admin_keypair.pubkey()));

    transaction.sign(
        &[&admin_keypair],
        rpc_client.get_latest_blockhash().unwrap(),
    );

    let sig = rpc_client
        .send_and_confirm_transaction(&transaction)
        .unwrap();

    println!("Transaction sent and confirmed: {}", sig);

    println!("Update config complete!");
    Ok(())
}
