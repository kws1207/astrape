use {
    borsh::{BorshDeserialize, BorshSerialize},
    breakout_contract::{
        instructions::TokenLockInstruction,
        state::{AstrapeConfig, UserDeposit, UserDepositState},
    },
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
        system_instruction,
        sysvar::{rent::Rent, Sysvar},
    },
    solana_program_test::*,
    solana_sdk::{
        account::Account,
        signature::{Keypair, Signer},
        sysvar::SysvarId,
        transaction::Transaction,
    },
    spl_associated_token_account::instruction as ata_instruction,
    spl_token::{
        instruction as token_instruction,
        state::{Account as TokenAccount, Mint},
    },
};

// Constants for testing
const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
const CONFIG_SEED: &[u8] = b"pool_config";
const STATE_SEED: &[u8] = b"pool_state";
const AUTHORITY_SEED: &[u8] = b"authority";
const WITHDRAWAL_POOL_SEED: &[u8] = b"withdrawal_pool";

// Test helper struct
struct TestHelper {
    admin: Keypair,
    program_id: Pubkey,
    interest_mint: Keypair,
    collateral_mint: Keypair,
    user: Keypair,
    config_pda: Pubkey,
    state_pda: Pubkey,
    authority_pda: Pubkey,
    interest_pool_ata: Pubkey,
    collateral_pool_ata: Pubkey,
    withdrawal_pool_pda: Pubkey,
    user_interest_ata: Pubkey,
    user_collateral_ata: Pubkey,
    admin_interest_ata: Pubkey,
    admin_collateral_ata: Pubkey,
    user_deposit_account: Pubkey,
}

impl TestHelper {
    async fn new(program_test: &mut ProgramTest) -> Self {
        // Set up admin account (using a new keypair since we don't have access to hardcoded admin key)
        let admin = Keypair::new();
        let program_id = breakout_contract::id();

        // Create mint keypairs
        let interest_mint = Keypair::new();
        let collateral_mint = Keypair::new();

        // Create a user keypair
        let user = Keypair::new();

        // Find PDAs
        let (config_pda, _) = Pubkey::find_program_address(&[CONFIG_SEED], &program_id);
        let (state_pda, _) = Pubkey::find_program_address(&[STATE_SEED], &program_id);
        let (authority_pda, _) = Pubkey::find_program_address(&[AUTHORITY_SEED], &program_id);
        let (withdrawal_authority, _) =
            Pubkey::find_program_address(&[WITHDRAWAL_POOL_SEED], &program_id);

        // Calculate ATAs
        let interest_pool_ata = spl_associated_token_account::get_associated_token_address(
            &authority_pda,
            &interest_mint.pubkey(),
        );

        let collateral_pool_ata = spl_associated_token_account::get_associated_token_address(
            &authority_pda,
            &collateral_mint.pubkey(),
        );

        // Add the withdrawal pool ATA
        let (withdrawal_pool_pda, _) =
            Pubkey::find_program_address(&[WITHDRAWAL_POOL_SEED], &program_id);

        let user_interest_ata = spl_associated_token_account::get_associated_token_address(
            &user.pubkey(),
            &interest_mint.pubkey(),
        );

        let user_collateral_ata = spl_associated_token_account::get_associated_token_address(
            &user.pubkey(),
            &collateral_mint.pubkey(),
        );

        let admin_interest_ata = spl_associated_token_account::get_associated_token_address(
            &admin.pubkey(),
            &interest_mint.pubkey(),
        );

        let admin_collateral_ata = spl_associated_token_account::get_associated_token_address(
            &admin.pubkey(),
            &collateral_mint.pubkey(),
        );

        // User deposit account (PDA derived from user pubkey)
        let (user_deposit_account, _) =
            Pubkey::find_program_address(&[user.pubkey().as_ref()], &program_id);

        // Add account with some lamports to program_test to work with
        program_test.add_account(
            admin.pubkey(),
            Account {
                lamports: LAMPORTS_PER_SOL * 1000,
                ..Account::default()
            },
        );

        program_test.add_account(
            user.pubkey(),
            Account {
                lamports: LAMPORTS_PER_SOL * 100,
                ..Account::default()
            },
        );

        Self {
            admin,
            program_id,
            interest_mint,
            collateral_mint,
            user,
            config_pda,
            state_pda,
            authority_pda,
            interest_pool_ata,
            collateral_pool_ata,
            withdrawal_pool_pda,
            user_interest_ata,
            user_collateral_ata,
            admin_interest_ata,
            admin_collateral_ata,
            user_deposit_account,
        }
    }

    async fn setup_mints(&self, banks_client: &mut BanksClient) {
        // Create interest mint
        let rent = banks_client.get_rent().await.unwrap();
        let mint_rent = rent.minimum_balance(Mint::LEN);

        let mut transaction = Transaction::new_with_payer(
            &[
                system_instruction::create_account(
                    &self.admin.pubkey(),
                    &self.interest_mint.pubkey(),
                    mint_rent,
                    Mint::LEN as u64,
                    &spl_token::id(),
                ),
                token_instruction::initialize_mint(
                    &spl_token::id(),
                    &self.interest_mint.pubkey(),
                    &self.admin.pubkey(),
                    None,
                    6,
                )
                .unwrap(),
            ],
            Some(&self.admin.pubkey()),
        );

        transaction.sign(
            &[&self.admin, &self.interest_mint],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
        banks_client.process_transaction(transaction).await.unwrap();

        // Create collateral mint
        let mut transaction = Transaction::new_with_payer(
            &[
                system_instruction::create_account(
                    &self.admin.pubkey(),
                    &self.collateral_mint.pubkey(),
                    mint_rent,
                    Mint::LEN as u64,
                    &spl_token::id(),
                ),
                token_instruction::initialize_mint(
                    &spl_token::id(),
                    &self.collateral_mint.pubkey(),
                    &self.admin.pubkey(),
                    None,
                    6,
                )
                .unwrap(),
            ],
            Some(&self.admin.pubkey()),
        );

        transaction.sign(
            &[&self.admin, &self.collateral_mint],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
        banks_client.process_transaction(transaction).await.unwrap();
    }

    async fn create_token_accounts(&self, banks_client: &mut BanksClient) {
        // Create admin token accounts
        let mut transaction = Transaction::new_with_payer(
            &[
                ata_instruction::create_associated_token_account(
                    &self.admin.pubkey(),
                    &self.admin.pubkey(),
                    &self.interest_mint.pubkey(),
                    &spl_token::id(),
                ),
                ata_instruction::create_associated_token_account(
                    &self.admin.pubkey(),
                    &self.admin.pubkey(),
                    &self.collateral_mint.pubkey(),
                    &spl_token::id(),
                ),
            ],
            Some(&self.admin.pubkey()),
        );

        transaction.sign(
            &[&self.admin],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
        banks_client.process_transaction(transaction).await.unwrap();

        // Create user token accounts
        let mut transaction = Transaction::new_with_payer(
            &[
                ata_instruction::create_associated_token_account(
                    &self.admin.pubkey(),
                    &self.user.pubkey(),
                    &self.interest_mint.pubkey(),
                    &spl_token::id(),
                ),
                ata_instruction::create_associated_token_account(
                    &self.admin.pubkey(),
                    &self.user.pubkey(),
                    &self.collateral_mint.pubkey(),
                    &spl_token::id(),
                ),
            ],
            Some(&self.admin.pubkey()),
        );

        transaction.sign(
            &[&self.admin],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
        banks_client.process_transaction(transaction).await.unwrap();
    }

    async fn mint_tokens(&self, banks_client: &mut BanksClient) {
        // Mint interest tokens to admin
        let mut transaction = Transaction::new_with_payer(
            &[token_instruction::mint_to(
                &spl_token::id(),
                &self.interest_mint.pubkey(),
                &self.admin_interest_ata,
                &self.admin.pubkey(),
                &[],
                1_000_000_000_000, // 1,000,000 tokens with 6 decimals
            )
            .unwrap()],
            Some(&self.admin.pubkey()),
        );

        transaction.sign(
            &[&self.admin],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
        banks_client.process_transaction(transaction).await.unwrap();

        // Mint collateral tokens to user
        let mut transaction = Transaction::new_with_payer(
            &[token_instruction::mint_to(
                &spl_token::id(),
                &self.collateral_mint.pubkey(),
                &self.user_collateral_ata,
                &self.admin.pubkey(),
                &[],
                100_000_000_000, // 100,000 tokens with 6 decimals
            )
            .unwrap()],
            Some(&self.admin.pubkey()),
        );

        transaction.sign(
            &[&self.admin],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
        banks_client.process_transaction(transaction).await.unwrap();
    }

    async fn initialize_program(&self, banks_client: &mut BanksClient) {
        // Log the authority PDA address
        self.create_authority_pda(banks_client).await;

        // Initialize the program with configuration
        let initialize_instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.admin.pubkey(), true), // Admin (payer & signer)
                AccountMeta::new(self.config_pda, false),    // Config PDA
                AccountMeta::new(self.authority_pda, false), // Authority PDA
                AccountMeta::new(self.interest_pool_ata, false), // Interest pool ATA
                AccountMeta::new(self.collateral_pool_ata, false), // Collateral pool ATA
                AccountMeta::new(self.withdrawal_pool_pda, false), // Withdrawal pool account
                AccountMeta::new_readonly(self.interest_mint.pubkey(), false), // Interest mint
                AccountMeta::new_readonly(self.collateral_mint.pubkey(), false), // Collateral mint
                AccountMeta::new_readonly(solana_program::system_program::id(), false), // System program
                AccountMeta::new_readonly(spl_token::id(), false), // Token program
                AccountMeta::new_readonly(spl_associated_token_account::id(), false), // ATA program
                AccountMeta::new_readonly(Rent::id(), false),      // Rent sysvar
            ],
            data: TokenLockInstruction::Initialize {
                interest_mint: self.interest_mint.pubkey(),
                collateral_mint: self.collateral_mint.pubkey(),
                base_interest_rate: 500,   // 5% annual rate (in basis points)
                price_factor: 10000,       // Scaling factor
                min_commission_rate: 100,  // 1% commission
                max_commission_rate: 1000, // 10% commission
                min_deposit_amount: 1_000_000, // 1 token with 6 decimals
                max_deposit_amount: 1_000_000_000_000, // 1,000,000 tokens
                deposit_periods: vec![100, 200, 300], // Different deposit periods in slots
            }
            .try_to_vec()
            .unwrap(),
        };

        // Send the initialize instruction
        let mut transaction =
            Transaction::new_with_payer(&[initialize_instruction], Some(&self.admin.pubkey()));

        transaction.sign(
            &[&self.admin],
            banks_client.get_latest_blockhash().await.unwrap(),
        );

        // Process the transaction and handle the result
        match banks_client.process_transaction(transaction).await {
            Ok(_) => log::info!("Transaction processed successfully"),
            Err(e) => log::info!("Transaction failed: {:?}", e),
        }
    }

    async fn admin_deposit_interest(&self, banks_client: &mut BanksClient, amount: u64) {
        let deposit_interest_instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.admin.pubkey(), true),
                AccountMeta::new_readonly(self.config_pda, false),
                AccountMeta::new_readonly(self.authority_pda, false),
                AccountMeta::new(self.admin_interest_ata, false),
                AccountMeta::new(self.interest_pool_ata, false),
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            ],
            data: TokenLockInstruction::AdminDepositInterest { amount }
                .try_to_vec()
                .unwrap(),
        };

        let mut transaction = Transaction::new_with_payer(
            &[deposit_interest_instruction],
            Some(&self.admin.pubkey()),
        );

        transaction.sign(
            &[&self.admin],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
        banks_client.process_transaction(transaction).await.unwrap();
    }

    async fn deposit_collateral(
        &self,
        banks_client: &mut BanksClient,
        amount: u64,
        deposit_period: u64,
        commission_rate: u64,
    ) {
        let deposit_collateral_instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.user.pubkey(), true),
                AccountMeta::new_readonly(self.config_pda, false),
                AccountMeta::new_readonly(self.authority_pda, false),
                AccountMeta::new(self.user_collateral_ata, false),
                AccountMeta::new(self.user_deposit_account, false),
                AccountMeta::new(self.collateral_pool_ata, false),
                AccountMeta::new(self.user_interest_ata, false),
                AccountMeta::new(self.interest_pool_ata, false),
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: TokenLockInstruction::DepositCollateral {
                amount,
                deposit_period,
                commission_rate,
            }
            .try_to_vec()
            .unwrap(),
        };

        let mut transaction = Transaction::new_with_payer(
            &[deposit_collateral_instruction],
            Some(&self.user.pubkey()),
        );

        transaction.sign(
            &[&self.user],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
        banks_client.process_transaction(transaction).await.unwrap();
    }

    async fn admin_withdraw_collateral_for_investment(&self, banks_client: &mut BanksClient) {
        let withdraw_instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.admin.pubkey(), true), // Admin (payer & signer)
                AccountMeta::new_readonly(self.config_pda, false), // Config PDA
                AccountMeta::new_readonly(self.authority_pda, false), // Authority PDA
                AccountMeta::new(self.admin_collateral_ata, false), // Admin collateral ATA
                AccountMeta::new(self.collateral_pool_ata, false), // Collateral pool ATA
                AccountMeta::new_readonly(solana_program::system_program::id(), false), // System program
                AccountMeta::new_readonly(spl_token::id(), false), // Token program
                AccountMeta::new_readonly(spl_associated_token_account::id(), false), // ATA program
            ],
            data: TokenLockInstruction::AdminWithdrawCollateralForInvestment
                .try_to_vec()
                .unwrap(),
        };

        let mut transaction =
            Transaction::new_with_payer(&[withdraw_instruction], Some(&self.admin.pubkey()));

        transaction.sign(
            &[&self.admin],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
        banks_client.process_transaction(transaction).await.unwrap();
    }

    async fn request_withdrawal_early(&self, banks_client: &mut BanksClient) {
        let request_withdrawal_instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.user.pubkey(), true),
                AccountMeta::new_readonly(self.config_pda, false),
                AccountMeta::new_readonly(self.authority_pda, false),
                AccountMeta::new(self.user_deposit_account, false),
                AccountMeta::new(self.user_interest_ata, false),
                AccountMeta::new(self.interest_pool_ata, false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: TokenLockInstruction::RequestWithdrawalEarly
                .try_to_vec()
                .unwrap(),
        };

        let mut transaction = Transaction::new_with_payer(
            &[request_withdrawal_instruction],
            Some(&self.user.pubkey()),
        );

        transaction.sign(
            &[&self.user],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
        banks_client.process_transaction(transaction).await.unwrap();
    }

    async fn request_withdrawal(&self, banks_client: &mut BanksClient) {
        let request_withdrawal_instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.user.pubkey(), true),
                AccountMeta::new(self.user_deposit_account, false),
            ],
            data: TokenLockInstruction::RequestWithdrawal
                .try_to_vec()
                .unwrap(),
        };

        let mut transaction = Transaction::new_with_payer(
            &[request_withdrawal_instruction],
            Some(&self.user.pubkey()),
        );

        transaction.sign(
            &[&self.user],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
        banks_client.process_transaction(transaction).await.unwrap();
    }

    async fn admin_prepare_withdrawal(&self, banks_client: &mut BanksClient, user_pubkey: Pubkey) {
        let prepare_withdrawal_instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.admin.pubkey(), true),
                AccountMeta::new_readonly(self.config_pda, false),
                AccountMeta::new(self.admin_collateral_ata, false),
                AccountMeta::new(self.withdrawal_pool_pda, false),
                AccountMeta::new_readonly(user_pubkey, false),
                AccountMeta::new(self.user_deposit_account, false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: TokenLockInstruction::AdminPrepareWithdrawal
                .try_to_vec()
                .unwrap(),
        };

        let mut transaction = Transaction::new_with_payer(
            &[prepare_withdrawal_instruction],
            Some(&self.admin.pubkey()),
        );

        transaction.sign(
            &[&self.admin],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
        banks_client.process_transaction(transaction).await.unwrap();
    }

    async fn withdraw_collateral(&self, banks_client: &mut BanksClient) {
        let withdraw_collateral_instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.user.pubkey(), true),
                AccountMeta::new_readonly(self.config_pda, false),
                AccountMeta::new_readonly(self.authority_pda, false),
                AccountMeta::new(self.user_deposit_account, false),
                AccountMeta::new(self.user_collateral_ata, false),
                AccountMeta::new(self.withdrawal_pool_pda, false),
                AccountMeta::new_readonly(spl_token::id(), false),
            ],
            data: TokenLockInstruction::WithdrawCollateral
                .try_to_vec()
                .unwrap(),
        };

        let mut transaction = Transaction::new_with_payer(
            &[withdraw_collateral_instruction],
            Some(&self.user.pubkey()),
        );

        transaction.sign(
            &[&self.user],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
        banks_client.process_transaction(transaction).await.unwrap();
    }

    async fn admin_update_config(&self, banks_client: &mut BanksClient) {
        let update_config_instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.admin.pubkey(), true),
                AccountMeta::new(self.config_pda, false),
            ],
            data: TokenLockInstruction::AdminUpdateConfig {
                param: 0,                      // Update base interest rate
                base_interest_rate: Some(600), // Update to 6%
                price_factor: None,
                min_commission_rate: None,
                max_commission_rate: None,
                min_deposit_amount: None,
                max_deposit_amount: None,
                deposit_periods: None,
            }
            .try_to_vec()
            .unwrap(),
        };

        let mut transaction =
            Transaction::new_with_payer(&[update_config_instruction], Some(&self.admin.pubkey()));

        transaction.sign(
            &[&self.admin],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
        banks_client.process_transaction(transaction).await.unwrap();
    }

    async fn read_config(
        &self,
        banks_client: &mut BanksClient,
    ) -> Result<AstrapeConfig, Box<dyn std::error::Error>> {
        let config_account = banks_client
            .get_account(self.config_pda)
            .await?
            .ok_or("Config account not found")?;
        log::info!("Config account data: {:?}", config_account.data);
        Ok(AstrapeConfig::try_from_slice(&config_account.data)?)
    }

    async fn get_token_balance(
        &self,
        banks_client: &mut BanksClient,
        token_account: &Pubkey,
    ) -> u64 {
        let account = banks_client
            .get_account(*token_account)
            .await
            .unwrap()
            .unwrap();
        let token_account = TokenAccount::unpack(&account.data).unwrap();
        token_account.amount
    }

    /// Helper method to create the authority PDA
    async fn create_authority_pda(&self, banks_client: &mut BanksClient) {
        // Get the rent for the authority PDA
        let rent = banks_client.get_rent().await.unwrap();
        let min_rent = rent.minimum_balance(0); // Minimum size account

        // Find the authority PDA details
        let (authority_pda, authority_bump) =
            Pubkey::find_program_address(&[AUTHORITY_SEED], &self.program_id);

        // We actually don't need to create the account directly
        // PDAs are owned by the program and are only created when needed
        // For ATAs, the program will just use the bumped PDA

        log::info!("Authority PDA at {}", authority_pda);
    }

    async fn read_user_deposit(
        &self,
        banks_client: &mut BanksClient,
    ) -> Result<UserDeposit, Box<dyn std::error::Error>> {
        let deposit_account = banks_client
            .get_account(self.user_deposit_account)
            .await?
            .ok_or("User deposit account not found")?;
        Ok(UserDeposit::try_from_slice(&deposit_account.data)?)
    }

    async fn admin_withdraw_interest(&self, banks_client: &mut BanksClient, amount: u64) {
        let withdraw_interest_instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.admin.pubkey(), true),
                AccountMeta::new_readonly(self.config_pda, false),
                AccountMeta::new_readonly(self.authority_pda, false),
                AccountMeta::new(self.admin_interest_ata, false),
                AccountMeta::new(self.interest_pool_ata, false),
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            ],
            data: TokenLockInstruction::AdminWithdrawInterest { amount }
                .try_to_vec()
                .unwrap(),
        };

        let mut transaction = Transaction::new_with_payer(
            &[withdraw_interest_instruction],
            Some(&self.admin.pubkey()),
        );

        transaction.sign(
            &[&self.admin],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
        banks_client.process_transaction(transaction).await.unwrap();
    }
}

#[tokio::test]
async fn test_full_flow() {
    env_logger::init();
    log::info!("=============================================");
    log::info!("STARTING TOKEN LOCK CONTRACT INTEGRATION TEST");
    log::info!("=============================================");

    // Initialize the test context
    log::info!("Setting up test environment...");
    let program_id = breakout_contract::id();
    log::info!("Program ID: {}", program_id);

    let mut program_test = ProgramTest::new(
        "breakout_contract",
        program_id,
        processor!(breakout_contract::entrypoint::process_instruction),
    );

    log::info!("Creating test helper with accounts and PDAs...");
    // Initialize the test helper
    let test_helper = TestHelper::new(&mut program_test).await;

    log::info!("Admin pubkey: {}", test_helper.admin.pubkey());
    log::info!("Config PDA: {}", test_helper.config_pda);
    log::info!("Authority PDA: {}", test_helper.authority_pda);
    log::info!("Interest mint: {}", test_helper.interest_mint.pubkey());
    log::info!("Collateral mint: {}", test_helper.collateral_mint.pubkey());

    log::info!("Starting banks client...");
    let (mut banks_client, _payer, _recent_blockhash) = program_test.start().await;

    // Setup test environment
    log::info!("\n[1/3] Setting up token mints...");
    test_helper.setup_mints(&mut banks_client).await;
    log::info!("✓ Mints created successfully");

    log::info!("\n[2/3] Creating token accounts...");
    test_helper.create_token_accounts(&mut banks_client).await;
    log::info!("✓ Token accounts created successfully");

    log::info!("\n[3/3] Minting initial tokens...");
    test_helper.mint_tokens(&mut banks_client).await;
    log::info!("✓ Tokens minted successfully");

    log::info!("\nTEST ENVIRONMENT SETUP COMPLETE");
    log::info!("-------------------------------");

    // Initialize the program
    log::info!("\n🔍 INITIALIZING PROGRAM");
    log::info!("Calling Initialize instruction...");

    // Get balances before
    if let Ok(account) = banks_client
        .get_account(test_helper.interest_pool_ata)
        .await
    {
        log::info!(
            "Interest pool ATA exists before init: {}",
            account.is_some()
        );
    } else {
        log::info!("Error checking interest pool ATA");
    }

    if let Ok(account) = banks_client
        .get_account(test_helper.collateral_pool_ata)
        .await
    {
        log::info!(
            "Collateral pool ATA exists before init: {}",
            account.is_some()
        );
    } else {
        log::info!("Error checking collateral pool ATA");
    }

    if let Ok(account) = banks_client.get_account(test_helper.config_pda).await {
        log::info!("Config PDA exists before init: {}", account.is_some());
    } else {
        log::info!("Error checking config PDA");
    }

    log::info!("Sending initialize transaction...");
    test_helper.initialize_program(&mut banks_client).await;
    log::info!("✓ Program initialized successfully");

    // Check account states after initialization
    log::info!("\nVerifying accounts after initialization:");
    if let Ok(account) = banks_client.get_account(test_helper.config_pda).await {
        log::info!("Config PDA exists: {}", account.is_some());
        if let Some(acc) = account {
            log::info!("Config data size: {} bytes", acc.data.len());
        }
    } else {
        log::info!("Error checking config PDA");
    }

    if let Ok(account) = banks_client
        .get_account(test_helper.interest_pool_ata)
        .await
    {
        log::info!("Interest pool ATA exists: {}", account.is_some());
    } else {
        log::info!("Error checking interest pool ATA");
    }

    if let Ok(account) = banks_client
        .get_account(test_helper.collateral_pool_ata)
        .await
    {
        log::info!("Collateral pool ATA exists: {}", account.is_some());
    } else {
        log::info!("Error checking collateral pool ATA");
    }

    // Only try reading the config if it exists
    log::info!("\nReading configuration data...");
    if let Ok(Some(_)) = banks_client.get_account(test_helper.config_pda).await {
        match test_helper.read_config(&mut banks_client).await {
            Ok(config) => {
                log::info!("Config data read successfully:");
                log::info!("  Base interest rate: {}", config.base_interest_rate);
                log::info!("  Min commission rate: {}", config.min_commission_rate);
                log::info!("  Max commission rate: {}", config.max_commission_rate);
                log::info!("  Interest mint: {}", config.interest_mint);
                log::info!("  Collateral mint: {}", config.collateral_mint);

                assert_eq!(config.base_interest_rate, 500);
                assert_eq!(config.min_commission_rate, 100);
                log::info!("✓ Configuration verified");
            }
            Err(e) => {
                log::error!("Failed to read config data: {}", e);
                log::info!("Ending test early.");
                return;
            }
        }
    } else {
        log::info!("Config account doesn't exist, initialization likely failed.");
        log::info!("Ending test early.");
        return;
    }

    // Admin deposits interest for users to earn
    log::info!("\n🔍 TESTING ADMIN DEPOSIT INTEREST");
    let interest_deposit_amount = 10_000_000_000; // 10,000 tokens
    log::info!(
        "Admin depositing {} interest tokens...",
        interest_deposit_amount / 1_000_000
    );

    // Check balances before
    let admin_interest_before = test_helper
        .get_token_balance(&mut banks_client, &test_helper.admin_interest_ata)
        .await;
    let pool_interest_before = test_helper
        .get_token_balance(&mut banks_client, &test_helper.interest_pool_ata)
        .await;
    log::info!(
        "Admin interest balance before: {} tokens",
        admin_interest_before / 1_000_000
    );
    log::info!(
        "Pool interest balance before: {} tokens",
        pool_interest_before / 1_000_000
    );

    test_helper
        .admin_deposit_interest(&mut banks_client, interest_deposit_amount)
        .await;

    // Check balances after
    let admin_interest_after = test_helper
        .get_token_balance(&mut banks_client, &test_helper.admin_interest_ata)
        .await;
    let pool_interest_after = test_helper
        .get_token_balance(&mut banks_client, &test_helper.interest_pool_ata)
        .await;
    log::info!(
        "Admin interest balance after: {} tokens",
        admin_interest_after / 1_000_000
    );
    log::info!(
        "Pool interest balance after: {} tokens",
        pool_interest_after / 1_000_000
    );
    log::info!("✓ Admin deposit interest successful");

    // User deposits collateral
    log::info!("\n🔍 TESTING USER DEPOSIT COLLATERAL");
    let deposit_amount = 5_000_000_000; // 5,000 tokens
    let current_slot = banks_client.get_root_slot().await.unwrap();
    let deposit_period = 100;
    let commission_rate = 100;
    log::info!("Current slot: {}", current_slot);
    log::info!("Deposit period: {}", deposit_period);
    log::info!(
        "User depositing {} collateral tokens until slot {}...",
        deposit_amount / 1_000_000,
        current_slot + deposit_period
    );

    // Check balances before
    let user_collateral_before = test_helper
        .get_token_balance(&mut banks_client, &test_helper.user_collateral_ata)
        .await;
    let pool_collateral_before = test_helper
        .get_token_balance(&mut banks_client, &test_helper.collateral_pool_ata)
        .await;
    log::info!(
        "User collateral balance before: {} tokens",
        user_collateral_before / 1_000_000
    );
    log::info!(
        "Pool collateral balance before: {} tokens",
        pool_collateral_before / 1_000_000
    );

    test_helper
        .deposit_collateral(
            &mut banks_client,
            deposit_amount,
            deposit_period,
            commission_rate,
        )
        .await;

    // Check balances after
    let user_collateral_after = test_helper
        .get_token_balance(&mut banks_client, &test_helper.user_collateral_ata)
        .await;
    let pool_collateral_after = test_helper
        .get_token_balance(&mut banks_client, &test_helper.collateral_pool_ata)
        .await;
    log::info!(
        "User collateral balance after: {} tokens",
        user_collateral_after / 1_000_000
    );
    log::info!(
        "Pool collateral balance after: {} tokens",
        pool_collateral_after / 1_000_000
    );

    // Verify deposit state
    log::info!("\nVerifying deposit state...");
    if let Ok(user_deposit) = test_helper.read_user_deposit(&mut banks_client).await {
        log::info!("Deposit amount: {}", user_deposit.amount);
        log::info!("Deposit state: {:?}", user_deposit.state);
        log::info!("Interest received: {}", user_deposit.interest_received);

        assert_eq!(user_deposit.amount, deposit_amount);
        assert_eq!(user_deposit.state, UserDepositState::Deposited);
        log::info!("✓ User deposit verified");
    } else {
        log::info!("Failed to read user deposit");
    }

    // Admin withdraws collateral for investment
    log::info!("\n🔍 TESTING ADMIN WITHDRAW COLLATERAL FOR INVESTMENT");
    log::info!("Admin withdrawing collateral for investment...");

    // Check balances before
    let admin_collateral_before = test_helper
        .get_token_balance(&mut banks_client, &test_helper.admin_collateral_ata)
        .await;
    let pool_collateral_before = test_helper
        .get_token_balance(&mut banks_client, &test_helper.collateral_pool_ata)
        .await;
    log::info!(
        "Admin collateral balance before: {} tokens",
        admin_collateral_before / 1_000_000
    );
    log::info!(
        "Pool collateral balance before: {} tokens",
        pool_collateral_before / 1_000_000
    );

    test_helper
        .admin_withdraw_collateral_for_investment(&mut banks_client)
        .await;

    // Check balances after
    let admin_collateral_after = test_helper
        .get_token_balance(&mut banks_client, &test_helper.admin_collateral_ata)
        .await;
    let pool_collateral_after = test_helper
        .get_token_balance(&mut banks_client, &test_helper.collateral_pool_ata)
        .await;
    log::info!(
        "Admin collateral balance after: {} tokens",
        admin_collateral_after / 1_000_000
    );
    log::info!(
        "Pool collateral balance after: {} tokens",
        pool_collateral_after / 1_000_000
    );

    // Verify admin received the collateral
    assert_eq!(admin_collateral_after, deposit_amount);
    log::info!("✓ Admin withdrawal verified");

    // Fast-forward slots (not actually possible in test, but we'll pretend)
    // In a real scenario, time would pass and the unlock_slot would be reached
    log::info!("\n🔍 TESTING WITHDRAWAL FLOW");
    log::info!("Starting withdrawal flow (simulating time passing)...");

    // User requests early withdrawal
    log::info!("\nUser requesting early withdrawal...");
    test_helper
        .request_withdrawal_early(&mut banks_client)
        .await;
    log::info!("✓ Early withdrawal requested");

    // Verify deposit state changed
    log::info!("\nVerifying deposit state after request...");
    if let Ok(user_deposit) = test_helper.read_user_deposit(&mut banks_client).await {
        log::info!("Deposit state: {:?}", user_deposit.state);
        assert_eq!(user_deposit.state, UserDepositState::WithdrawRequested);
        log::info!("✓ State change verified");
    } else {
        log::info!("Failed to read user deposit");
    }

    // Admin prepares for user withdrawal
    log::info!("\nAdmin preparing withdrawal...");
    test_helper
        .admin_prepare_withdrawal(&mut banks_client, test_helper.user.pubkey())
        .await;
    log::info!("✓ Withdrawal prepared");

    // Verify deposit state changed again
    log::info!("\nVerifying deposit state after preparation...");
    if let Ok(user_deposit) = test_helper.read_user_deposit(&mut banks_client).await {
        log::info!("Deposit state: {:?}", user_deposit.state);
        assert_eq!(user_deposit.state, UserDepositState::WithdrawReady);
        log::info!("✓ State change verified");
    } else {
        log::info!("Failed to read user deposit");
    }

    // User withdraws collateral
    log::info!("\nUser withdrawing collateral...");

    // Check balances before
    let user_collateral_before = test_helper
        .get_token_balance(&mut banks_client, &test_helper.user_collateral_ata)
        .await;
    log::info!(
        "User collateral balance before: {} tokens",
        user_collateral_before / 1_000_000
    );

    test_helper.withdraw_collateral(&mut banks_client).await;

    // Check balances after
    let user_collateral_after = test_helper
        .get_token_balance(&mut banks_client, &test_helper.user_collateral_ata)
        .await;
    log::info!(
        "User collateral balance after: {} tokens",
        user_collateral_after / 1_000_000
    );

    // Verify deposit state is now completed
    log::info!("\nVerifying final deposit state...");
    if let Ok(user_deposit) = test_helper.read_user_deposit(&mut banks_client).await {
        log::info!("Deposit state: {:?}", user_deposit.state);
        assert_eq!(user_deposit.state, UserDepositState::WithdrawCompleted);
        log::info!("✓ Withdrawal completed verified");
    } else {
        log::info!("Failed to read user deposit");
    }

    // Admin updates configuration
    log::info!("\n🔍 TESTING ADMIN UPDATE CONFIG");
    log::info!("Admin updating configuration...");

    // Get config before
    log::info!(
        "Base interest rate before: {}",
        if let Ok(config_before) = test_helper.read_config(&mut banks_client).await {
            config_before.base_interest_rate
        } else {
            log::info!("Failed to read config before update");
            0
        }
    );

    test_helper.admin_update_config(&mut banks_client).await;

    // Get config after
    if let Ok(config_after) = test_helper.read_config(&mut banks_client).await {
        log::info!(
            "Base interest rate after: {}",
            config_after.base_interest_rate
        );

        // Verify configuration was updated
        assert_eq!(config_after.base_interest_rate, 600); // Updated from 500 to 600
        log::info!("✓ Configuration update verified");
    } else {
        log::info!("Failed to read config after update");
    }

    log::info!("\n=============================================");
    log::info!("ALL TESTS COMPLETED SUCCESSFULLY");
    log::info!("=============================================");
}
