use anyhow::{anyhow, Result};
use astrape_admin_utils::{INTEREST_MINT, PROGRAM_ID};
use borsh::BorshSerialize;
use breakout_contract::{
    instructions::TokenLockInstruction,
    processor::{AUTHORITY_SEED, CONFIG_SEED},
};
use clap::Parser;
use solana_client::rpc_client::RpcClient;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{read_keypair_file, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
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

    let interest_mint = Pubkey::from_str(INTEREST_MINT)?;

    // Find PDAs
    let (config_pda, _) = Pubkey::find_program_address(&[CONFIG_SEED], &program_id);
    let (authority_pda, _) = Pubkey::find_program_address(&[AUTHORITY_SEED], &program_id);

    println!("Config PDA: {}", config_pda);
    println!("Authority PDA: {}", authority_pda);

    // Get associated token accounts
    let admin_interest_ata = get_associated_token_address(&admin_keypair.pubkey(), &interest_mint);
    let interest_pool_ata = get_associated_token_address(&authority_pda, &interest_mint);

    println!("Admin Interest ATA: {}", admin_interest_ata);
    println!("Interest Pool ATA: {}", interest_pool_ata);

    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_keypair.pubkey(), true),
            AccountMeta::new(config_pda, false),
            AccountMeta::new(authority_pda, false),
            AccountMeta::new(admin_interest_ata, false),
            AccountMeta::new(interest_pool_ata, false),
            AccountMeta::new(system_program::ID, false),
            AccountMeta::new(spl_token::id(), false),
            AccountMeta::new(spl_associated_token_account::id(), false),
        ],
        data: TokenLockInstruction::AdminDepositInterest {
            amount: 100_000_000_000,
        }
        .try_to_vec()
        .unwrap(),
    };

    let mut transaction =
        Transaction::new_with_payer(&[instruction], Some(&admin_keypair.pubkey()));

    transaction.sign(
        &[&admin_keypair],
        rpc_client.get_latest_blockhash().unwrap(),
    );

    let sig = rpc_client
        .send_and_confirm_transaction(&transaction)
        .unwrap();

    println!("Transaction sent and confirmed: {}", sig);

    println!("Admin deposit interest complete!");
    Ok(())
}
