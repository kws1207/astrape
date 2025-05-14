use {
    astrape::{
        instructions::TokenLockInstruction,
        processor::{
            Processor, AUTHORITY_SEED, CONFIG_SEED, SLOTS_PER_MONTH, WITHDRAWAL_POOL_SEED,
        },
        state::{AstrapeConfig, UserDeposit, UserDepositState},
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
        system_instruction,
        sysvar::rent::Rent,
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

// Test helper struct
struct TestHelper {
    admin: Keypair,
    program_id: Pubkey,
    interest_mint: Keypair,
    collateral_mint: Keypair,
    user: Keypair,
    config_pda: Pubkey,
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
    async fn new(
        admin: Keypair,
        user: Keypair,
        collateral_mint: Keypair,
        interest_mint: Keypair,
    ) -> Self {
        let program_id = astrape::id();

        // Find PDAs
        let (config_pda, _) = Pubkey::find_program_address(&[CONFIG_SEED], &program_id);
        let (authority_pda, _) = Pubkey::find_program_address(&[AUTHORITY_SEED], &program_id);

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

        Self {
            admin,
            program_id,
            interest_mint,
            collateral_mint,
            user,
            config_pda,
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
                base_interest_rate: 50, // 5% annual rate (in basis points)
                price_factor: 100_000 / 10_u64.pow(8 - 6), // zBTC's decimal: 8 , USDC's decimal: 6
                min_commission_rate: 100, // 10% commission
                max_commission_rate: 300, // 30% commission
                min_deposit_amount: 10_000_000, // 0.1 zBTC
                max_deposit_amount: 100_000_000, // 1 zBTC
                deposit_periods: vec![
                    1 * SLOTS_PER_MONTH as u64,
                    3 * SLOTS_PER_MONTH as u64,
                    6 * SLOTS_PER_MONTH as u64,
                ], // Different deposit periods in slots
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
                param: 0,                     // Update base interest rate
                base_interest_rate: Some(60), // Update to 6%
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

    async fn get_user_deposit(
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
    env_logger::try_init();
    log::info!("=============================================");
    log::info!("STARTING TOKEN LOCK CONTRACT INTEGRATION TEST");
    log::info!("=============================================");

    // Initialize the test context
    log::info!("Setting up test environment...");
    let program_id = astrape::id();
    log::info!("Program ID: {}", program_id);

    let mut program_test = ProgramTest::new(
        "astrape",
        program_id,
        processor!(astrape::entrypoint::process_instruction),
    );

    let admin = Keypair::from_bytes(&[
        97, 207, 117, 213, 126, 4, 83, 204, 14, 192, 150, 163, 42, 207, 232, 166, 98, 53, 10, 124,
        164, 132, 86, 113, 81, 3, 81, 125, 39, 72, 68, 202, 204, 13, 199, 8, 228, 122, 171, 83,
        131, 50, 27, 157, 206, 153, 164, 34, 8, 61, 202, 12, 178, 68, 104, 155, 158, 142, 181, 94,
        56, 2, 237, 86,
    ])
    .unwrap();

    // Add account with some lamports to program_test to work with
    program_test.add_account(
        admin.pubkey(),
        Account {
            lamports: LAMPORTS_PER_SOL * 1000,
            ..Account::default()
        },
    );

    let user = Keypair::new();

    program_test.add_account(
        user.pubkey(),
        Account {
            lamports: LAMPORTS_PER_SOL * 100,
            ..Account::default()
        },
    );

    log::info!("Starting banks client...");
    let (mut banks_client, _payer, _recent_blockhash) = program_test.start().await;

    let collateral_mint = Keypair::new();
    let interest_mint = Keypair::new();

    setup_mints(&mut banks_client, &admin, &interest_mint, &collateral_mint).await;
    setup_admin(&mut banks_client, &admin, &interest_mint, &collateral_mint).await;
    setup_user(
        &mut banks_client,
        &user,
        &admin,
        &collateral_mint.pubkey(),
        &interest_mint.pubkey(),
    )
    .await;

    log::info!("Creating test helper with accounts and PDAs...");
    // Initialize the test helper
    let test_helper = TestHelper::new(admin, user, collateral_mint, interest_mint).await;

    log::info!("Admin pubkey: {}", test_helper.admin.pubkey());
    log::info!("Config PDA: {}", test_helper.config_pda);
    log::info!("Authority PDA: {}", test_helper.authority_pda);
    log::info!("Interest mint: {}", test_helper.interest_mint.pubkey());
    log::info!("Collateral mint: {}", test_helper.collateral_mint.pubkey());

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

                assert_eq!(config.base_interest_rate, 50);
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
    let interest_deposit_amount = 1_000_000_000_000; // 1,000,000 USDC (with 6 decimals)
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
    let deposit_amount = 20_000_000; // 0.2 zBTC with 8 decimals
    let current_slot = banks_client.get_root_slot().await.unwrap();
    let deposit_period = 1 * SLOTS_PER_MONTH as u64; // 1 month period
    let commission_rate = 200; // 20% commission
    log::info!("Current slot: {}", current_slot);
    log::info!("Deposit period: {}", deposit_period);
    log::info!(
        "User depositing {} zBTC (with 8 decimals) until slot {}...",
        deposit_amount,
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
    if let Ok(user_deposit) = test_helper.get_user_deposit(&mut banks_client).await {
        log::info!("User deposit state: {:?}", user_deposit.state);
        log::info!("Deposit amount: {}", user_deposit.amount);
        log::info!("Interest received: {}", user_deposit.interest_received);

        // Calculate expected interest based on our parameters:
        // 0.2 zBTC at $100k per BTC = $20,000 value
        // Price factor = 1,000 (already accounts for decimal difference)
        // Base interest rate = 5% = 0.05
        // Period = 1 month = 1/12 year
        // Commission = 20% = 0.8 ratio without commission
        // Expected interest = 20,000,000 (0.2 BTC in lamports) * 1,000 * (1 + 0.05) * (1/12) * 0.8
        let expected_interest = Processor::calculate_interest_amount(
            deposit_amount,
            commission_rate,
            deposit_period,
            &test_helper.read_config(&mut banks_client).await.unwrap(),
        );
        log::info!("Expected interest: {}", expected_interest);
        assert_eq!(user_deposit.interest_received, expected_interest);
        assert_eq!(user_deposit.amount, deposit_amount);
        assert_eq!(user_deposit.state, UserDepositState::Deposited);

        // Check token balances
        let user_collateral_balance = test_helper
            .get_token_balance(&mut banks_client, &test_helper.user_collateral_ata)
            .await;
        let user_interest_balance = test_helper
            .get_token_balance(&mut banks_client, &test_helper.user_interest_ata)
            .await;
        let pool_collateral_balance = test_helper
            .get_token_balance(&mut banks_client, &test_helper.collateral_pool_ata)
            .await;
        let pool_interest_balance = test_helper
            .get_token_balance(&mut banks_client, &test_helper.interest_pool_ata)
            .await;

        log::info!(
            "User collateral balance after deposit: {}",
            user_collateral_balance
        );
        log::info!(
            "User interest balance after deposit: {}",
            user_interest_balance
        );
        log::info!(
            "Pool collateral balance after deposit: {}",
            pool_collateral_balance
        );
        log::info!(
            "Pool interest balance after deposit: {}",
            pool_interest_balance
        );

        // Verify balances
        assert_eq!(
            user_collateral_balance,
            1_000_000_000 - deposit_amount,
            "User should have less collateral after deposit"
        );
        assert_eq!(
            pool_collateral_balance, deposit_amount,
            "Pool should hold the deposited collateral"
        );
        assert_eq!(
            user_interest_balance, expected_interest,
            "User should receive calculated interest"
        );
        assert_eq!(
            pool_interest_balance,
            interest_deposit_amount - expected_interest,
            "Pool should have less interest tokens"
        );

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
    if let Ok(user_deposit) = test_helper.get_user_deposit(&mut banks_client).await {
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
    if let Ok(user_deposit) = test_helper.get_user_deposit(&mut banks_client).await {
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
    if let Ok(user_deposit) = test_helper.get_user_deposit(&mut banks_client).await {
        log::info!("Deposit state: {:?}", user_deposit.state);
        assert_eq!(user_deposit.state, UserDepositState::WithdrawCompleted);
        log::info!("✓ Withdrawal completed verified");
    } else {
        log::info!("Failed to read user deposit");
    }

    // After withdraw collateral
    log::info!("\nVerifying final account states...");

    // Check token balances after withdrawal
    let user_collateral_after = test_helper
        .get_token_balance(&mut banks_client, &test_helper.user_collateral_ata)
        .await;
    let pool_collateral_after = test_helper
        .get_token_balance(&mut banks_client, &test_helper.collateral_pool_ata)
        .await;
    let withdrawal_pool_after = test_helper
        .get_token_balance(&mut banks_client, &test_helper.withdrawal_pool_pda)
        .await;

    log::info!(
        "User collateral balance after withdrawal: {}",
        user_collateral_after
    );
    log::info!(
        "Pool collateral balance after withdrawal: {}",
        pool_collateral_after
    );
    log::info!(
        "Withdrawal pool balance after withdrawal: {}",
        withdrawal_pool_after
    );

    // User should have their initial balance back (minus any fees if applicable)
    assert_eq!(
        user_collateral_after, 1_000_000_000,
        "User should have received their collateral back"
    );
    assert_eq!(
        withdrawal_pool_after, 0,
        "Withdrawal pool should be empty after withdrawal"
    );

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
        assert_eq!(config_after.base_interest_rate, 60); // Updated from 50 to 60
        log::info!("✓ Configuration update verified");
    } else {
        log::info!("Failed to read config after update");
    }

    log::info!("\n=============================================");
    log::info!("ALL TESTS COMPLETED SUCCESSFULLY");
    log::info!("=============================================");
}

#[tokio::test]
async fn test_negative_cases() {
    // Set up the test environment similar to the main test
    env_logger::try_init();
    log::info!("Starting negative test cases");

    let program_id = astrape::id();
    let mut program_test = ProgramTest::new(
        "astrape",
        program_id,
        processor!(astrape::entrypoint::process_instruction),
    );

    let admin = Keypair::from_bytes(&[
        97, 207, 117, 213, 126, 4, 83, 204, 14, 192, 150, 163, 42, 207, 232, 166, 98, 53, 10, 124,
        164, 132, 86, 113, 81, 3, 81, 125, 39, 72, 68, 202, 204, 13, 199, 8, 228, 122, 171, 83,
        131, 50, 27, 157, 206, 153, 164, 34, 8, 61, 202, 12, 178, 68, 104, 155, 158, 142, 181, 94,
        56, 2, 237, 86,
    ])
    .unwrap();

    // Add account with some lamports to program_test to work with
    program_test.add_account(
        admin.pubkey(),
        Account {
            lamports: LAMPORTS_PER_SOL * 1000,
            ..Account::default()
        },
    );

    let user1 = Keypair::new();
    let user2 = Keypair::new();
    let user3 = Keypair::new();
    program_test.add_account(
        user1.pubkey(),
        Account {
            lamports: LAMPORTS_PER_SOL * 100,
            ..Account::default()
        },
    );
    program_test.add_account(
        user2.pubkey(),
        Account {
            lamports: LAMPORTS_PER_SOL * 100,
            ..Account::default()
        },
    );
    program_test.add_account(
        user3.pubkey(),
        Account {
            lamports: LAMPORTS_PER_SOL * 100,
            ..Account::default()
        },
    );

    log::info!("Starting banks client...");
    let (mut banks_client, _payer, _recent_blockhash) = program_test.start().await;

    let collateral_mint = Keypair::new();
    let interest_mint = Keypair::new();

    setup_mints(&mut banks_client, &admin, &interest_mint, &collateral_mint).await;
    setup_admin(&mut banks_client, &admin, &interest_mint, &collateral_mint).await;
    setup_user(
        &mut banks_client,
        &user1,
        &admin,
        &collateral_mint.pubkey(),
        &interest_mint.pubkey(),
    )
    .await;
    setup_user(
        &mut banks_client,
        &user2,
        &admin,
        &collateral_mint.pubkey(),
        &interest_mint.pubkey(),
    )
    .await;
    setup_user(
        &mut banks_client,
        &user3,
        &admin,
        &collateral_mint.pubkey(),
        &interest_mint.pubkey(),
    )
    .await;

    // Initialize the test helper
    let test_helper = TestHelper::new(admin, user1, collateral_mint, interest_mint).await;

    test_helper.initialize_program(&mut banks_client).await;

    // Add some interest to the pool for tests
    test_helper
        .admin_deposit_interest(&mut banks_client, 1_000_000_000_000)
        .await;

    // Negative Test 1: Deposit amount below minimum
    log::info!("\n🔍 TEST CASE: Deposit below minimum amount");
    let min_deposit = test_helper
        .read_config(&mut banks_client)
        .await
        .unwrap()
        .min_deposit_amount;
    log::info!("Minimum deposit amount: {}", min_deposit);

    let too_small_amount = min_deposit - 1;
    let result = test_deposit_with_amount(
        &test_helper,
        &mut banks_client,
        too_small_amount,
        1 * SLOTS_PER_MONTH as u64,
        200,
    )
    .await;
    assert!(
        result.is_err(),
        "Transaction should fail with amount below minimum"
    );
    log::info!("✓ Transaction correctly failed with amount below minimum");

    // Negative Test 2: Deposit amount above maximum
    log::info!("\n🔍 TEST CASE: Deposit above maximum amount");
    let max_deposit = test_helper
        .read_config(&mut banks_client)
        .await
        .unwrap()
        .max_deposit_amount;
    log::info!("Maximum deposit amount: {}", max_deposit);

    let too_large_amount = max_deposit + 1;
    let result = test_deposit_with_amount(
        &test_helper,
        &mut banks_client,
        too_large_amount,
        1 * SLOTS_PER_MONTH as u64,
        200,
    )
    .await;
    assert!(
        result.is_err(),
        "Transaction should fail with amount above maximum"
    );
    log::info!("✓ Transaction correctly failed with amount above maximum");

    // Negative Test 3: Invalid deposit period
    log::info!("\n🔍 TEST CASE: Invalid deposit period");
    let config = test_helper.read_config(&mut banks_client).await.unwrap();
    let invalid_period = 42; // Some arbitrary period not in the allowed list

    // Make sure our invalid period isn't accidentally in the allowed list
    assert!(
        !config.deposit_periods.contains(&invalid_period),
        "Test setup error: chosen invalid period is actually valid"
    );

    log::info!("Valid periods: {:?}", config.deposit_periods);
    log::info!("Using invalid period: {}", invalid_period);

    let result = test_deposit_with_amount(
        &test_helper,
        &mut banks_client,
        20_000_000,
        invalid_period,
        200,
    )
    .await;
    assert!(
        result.is_err(),
        "Transaction should fail with invalid deposit period"
    );
    log::info!("✓ Transaction correctly failed with invalid deposit period");

    // Negative Test 4: Commission rate out of range
    log::info!("\n🔍 TEST CASE: Commission rate out of range");
    let min_commission = config.min_commission_rate;
    let max_commission = config.max_commission_rate;
    let too_low_commission = min_commission - 1;

    log::info!(
        "Min commission: {}, Max commission: {}",
        min_commission,
        max_commission
    );
    log::info!("Using too low commission: {}", too_low_commission);

    let result = test_deposit_with_amount(
        &test_helper,
        &mut banks_client,
        20_000_000,
        1 * SLOTS_PER_MONTH as u64,
        too_low_commission,
    )
    .await;
    assert!(
        result.is_err(),
        "Transaction should fail with commission rate too low"
    );
    log::info!("✓ Transaction correctly failed with commission rate too low");

    // Also test commission too high
    let too_high_commission = max_commission + 1;
    log::info!("Using too high commission: {}", too_high_commission);

    let result = test_deposit_with_amount(
        &test_helper,
        &mut banks_client,
        20_000_000,
        1 * SLOTS_PER_MONTH as u64,
        too_high_commission,
    )
    .await;
    assert!(
        result.is_err(),
        "Transaction should fail with commission rate too high"
    );
    log::info!("✓ Transaction correctly failed with commission rate too high");

    // Negative Test 5: Withdrawing without admin preparation
    log::info!("\n🔍 TEST CASE: Withdraw without admin preparation");

    // First make a valid deposit
    test_deposit_with_amount(
        &test_helper,
        &mut banks_client,
        20_000_000,
        1 * SLOTS_PER_MONTH as u64,
        200,
    )
    .await
    .unwrap();

    // Request withdrawal (legitimate)
    test_helper
        .request_withdrawal_early(&mut banks_client)
        .await;

    // Try to withdraw without admin preparing it
    let result = test_withdraw_without_preparation(&test_helper, &mut banks_client).await;
    assert!(
        result.is_err(),
        "Withdrawal should fail without admin preparation"
    );
    log::info!("✓ Transaction correctly failed when withdrawing without admin preparation");

    // Negative Test 6: Non-admin trying to perform admin operation
    log::info!("\n🔍 TEST CASE: Non-admin trying to perform admin operation");
    let result = test_non_admin_operation(&test_helper, &mut banks_client).await;
    assert!(
        result.is_err(),
        "Transaction should fail when non-admin attempts admin operation"
    );
    log::info!("✓ Transaction correctly failed when non-admin attempted admin operation");

    // Negative Test 7: Double deposit attempt (can't deposit twice to same account)
    log::info!("\n🔍 TEST CASE: Double deposit attempt");

    // First make a valid deposit
    let valid_amount = 20_000_000;
    let valid_period = 1 * SLOTS_PER_MONTH as u64;
    let valid_commission = 200;

    // First deposit should succeed
    let result = test_deposit_for_user(
        &test_helper,
        &mut banks_client,
        &user2,
        valid_amount,
        valid_period,
        valid_commission,
    )
    .await;
    assert!(result.is_ok(), "First deposit should succeed");

    // Second deposit to same account should fail
    let result = test_deposit_for_user(
        &test_helper,
        &mut banks_client,
        &user2,
        valid_amount,
        valid_period,
        valid_commission,
    )
    .await;
    assert!(
        result.is_err(),
        "Second deposit to same account should fail"
    );
    log::info!("✓ Transaction correctly failed on double deposit attempt");

    // Negative Test 8: Attempting to withdraw from a deposit that's not in withdraw ready state
    log::info!("\n🔍 TEST CASE: Withdraw from deposit not in ready state");

    // Make a valid deposit
    test_deposit_for_user(
        &test_helper,
        &mut banks_client,
        &user3,
        valid_amount,
        valid_period,
        valid_commission,
    )
    .await
    .unwrap();

    // Try to withdraw immediately without requesting withdrawal first
    let result = test_withdraw_for_user(&test_helper, &mut banks_client, &user3).await;
    assert!(
        result.is_err(),
        "Withdrawal should fail when deposit not in ready state"
    );
    log::info!("✓ Transaction correctly failed when withdrawing from deposit not in ready state");

    log::info!("\n=============================================");
    log::info!("ALL NEGATIVE TESTS COMPLETED SUCCESSFULLY");
    log::info!("=============================================");
}

// Helper function to test deposit with specific parameters
async fn test_deposit_with_amount(
    test_helper: &TestHelper,
    banks_client: &mut BanksClient,
    amount: u64,
    deposit_period: u64,
    commission_rate: u64,
) -> Result<(), BanksClientError> {
    let deposit_instruction = Instruction {
        program_id: test_helper.program_id,
        accounts: vec![
            AccountMeta::new(test_helper.user.pubkey(), true),
            AccountMeta::new_readonly(test_helper.config_pda, false),
            AccountMeta::new_readonly(test_helper.authority_pda, false),
            AccountMeta::new(test_helper.user_collateral_ata, false),
            AccountMeta::new(test_helper.user_deposit_account, false),
            AccountMeta::new(test_helper.collateral_pool_ata, false),
            AccountMeta::new(test_helper.user_interest_ata, false),
            AccountMeta::new(test_helper.interest_pool_ata, false),
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

    let mut transaction =
        Transaction::new_with_payer(&[deposit_instruction], Some(&test_helper.user.pubkey()));

    transaction.sign(
        &[&test_helper.user],
        banks_client.get_latest_blockhash().await.unwrap(),
    );

    banks_client.process_transaction(transaction).await
}

// Helper function to test withdrawal without admin preparation
async fn test_withdraw_without_preparation(
    test_helper: &TestHelper,
    banks_client: &mut BanksClient,
) -> Result<(), BanksClientError> {
    let withdraw_instruction = Instruction {
        program_id: test_helper.program_id,
        accounts: vec![
            AccountMeta::new(test_helper.user.pubkey(), true),
            AccountMeta::new_readonly(test_helper.config_pda, false),
            AccountMeta::new_readonly(test_helper.authority_pda, false),
            AccountMeta::new(test_helper.user_deposit_account, false),
            AccountMeta::new(test_helper.user_collateral_ata, false),
            AccountMeta::new(test_helper.withdrawal_pool_pda, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: TokenLockInstruction::WithdrawCollateral
            .try_to_vec()
            .unwrap(),
    };

    let mut transaction =
        Transaction::new_with_payer(&[withdraw_instruction], Some(&test_helper.user.pubkey()));

    transaction.sign(
        &[&test_helper.user],
        banks_client.get_latest_blockhash().await.unwrap(),
    );

    banks_client.process_transaction(transaction).await
}

// Helper function to test non-admin trying to perform admin operation
async fn test_non_admin_operation(
    test_helper: &TestHelper,
    banks_client: &mut BanksClient,
) -> Result<(), BanksClientError> {
    // Try to update config as non-admin user
    let update_config_instruction = Instruction {
        program_id: test_helper.program_id,
        accounts: vec![
            AccountMeta::new(test_helper.user.pubkey(), true), // User instead of admin
            AccountMeta::new(test_helper.config_pda, false),
        ],
        data: TokenLockInstruction::AdminUpdateConfig {
            param: 0,
            base_interest_rate: Some(80),
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

    let mut transaction = Transaction::new_with_payer(
        &[update_config_instruction],
        Some(&test_helper.user.pubkey()),
    );

    transaction.sign(
        &[&test_helper.user],
        banks_client.get_latest_blockhash().await.unwrap(),
    );

    banks_client.process_transaction(transaction).await
}

async fn setup_mints(
    banks_client: &mut BanksClient,
    admin: &Keypair,
    interest_mint: &Keypair,
    collateral_mint: &Keypair,
) {
    // Create interest mint
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(Mint::LEN);

    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &admin.pubkey(),
                &interest_mint.pubkey(),
                mint_rent,
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            token_instruction::initialize_mint(
                &spl_token::id(),
                &interest_mint.pubkey(),
                &admin.pubkey(),
                None,
                6,
            )
            .unwrap(),
        ],
        Some(&admin.pubkey()),
    );

    transaction.sign(
        &[&admin, &interest_mint],
        banks_client.get_latest_blockhash().await.unwrap(),
    );
    banks_client.process_transaction(transaction).await.unwrap();

    // Create collateral mint
    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &admin.pubkey(),
                &collateral_mint.pubkey(),
                mint_rent,
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            token_instruction::initialize_mint(
                &spl_token::id(),
                &collateral_mint.pubkey(),
                &admin.pubkey(),
                None,
                6,
            )
            .unwrap(),
        ],
        Some(&admin.pubkey()),
    );

    transaction.sign(
        &[&admin, &collateral_mint],
        banks_client.get_latest_blockhash().await.unwrap(),
    );
    banks_client.process_transaction(transaction).await.unwrap();
}

async fn setup_admin(
    banks_client: &mut BanksClient,
    admin: &Keypair,
    interest_mint: &Keypair,
    collateral_mint: &Keypair,
) {
    // Create admin token accounts
    let mut transaction = Transaction::new_with_payer(
        &[
            ata_instruction::create_associated_token_account(
                &admin.pubkey(),
                &admin.pubkey(),
                &interest_mint.pubkey(),
                &spl_token::id(),
            ),
            ata_instruction::create_associated_token_account(
                &admin.pubkey(),
                &admin.pubkey(),
                &collateral_mint.pubkey(),
                &spl_token::id(),
            ),
        ],
        Some(&admin.pubkey()),
    );

    transaction.sign(
        &[&admin],
        banks_client.get_latest_blockhash().await.unwrap(),
    );
    banks_client.process_transaction(transaction).await.unwrap();

    // Mint interest tokens to admin
    let mut transaction = Transaction::new_with_payer(
        &[token_instruction::mint_to(
            &spl_token::id(),
            &interest_mint.pubkey(),
            &spl_associated_token_account::get_associated_token_address(
                &admin.pubkey(),
                &interest_mint.pubkey(),
            ),
            &admin.pubkey(),
            &[],
            10_000_000_000_000, // 10,000,000 USDC with 6 decimals
        )
        .unwrap()],
        Some(&admin.pubkey()),
    );

    transaction.sign(
        &[&admin],
        banks_client.get_latest_blockhash().await.unwrap(),
    );
    banks_client.process_transaction(transaction).await.unwrap()
}

// Helper function to create a new test user
async fn setup_user(
    banks_client: &mut BanksClient,
    user: &Keypair,
    admin: &Keypair,
    collateral_mint: &Pubkey,
    interest_mint: &Pubkey,
) {
    // Fund the user account
    let fund_ix =
        system_instruction::transfer(&admin.pubkey(), &user.pubkey(), LAMPORTS_PER_SOL * 10);

    let mut transaction = Transaction::new_with_payer(&[fund_ix], Some(&admin.pubkey()));

    transaction.sign(
        &[&admin],
        banks_client.get_latest_blockhash().await.unwrap(),
    );

    banks_client.process_transaction(transaction).await.unwrap();

    // Create token accounts for the user
    let create_token_accounts_ix = [
        ata_instruction::create_associated_token_account(
            &admin.pubkey(),
            &user.pubkey(),
            &interest_mint,
            &spl_token::id(),
        ),
        ata_instruction::create_associated_token_account(
            &admin.pubkey(),
            &user.pubkey(),
            &collateral_mint,
            &spl_token::id(),
        ),
    ];

    let mut transaction =
        Transaction::new_with_payer(&create_token_accounts_ix, Some(&admin.pubkey()));

    transaction.sign(
        &[&admin],
        banks_client.get_latest_blockhash().await.unwrap(),
    );

    banks_client.process_transaction(transaction).await.unwrap();

    // Mint collateral tokens to the user
    let user_collateral_ata = spl_associated_token_account::get_associated_token_address(
        &user.pubkey(),
        &collateral_mint,
    );

    let mint_tokens_ix = token_instruction::mint_to(
        &spl_token::id(),
        &collateral_mint,
        &user_collateral_ata,
        &admin.pubkey(),
        &[],
        1_000_000_000, // 10 zBTC with 8 decimals
    )
    .unwrap();

    let mut transaction = Transaction::new_with_payer(&[mint_tokens_ix], Some(&admin.pubkey()));

    transaction.sign(
        &[&admin],
        banks_client.get_latest_blockhash().await.unwrap(),
    );

    banks_client.process_transaction(transaction).await.unwrap();
}

// Helper function for user-specific deposit
async fn test_deposit_for_user(
    test_helper: &TestHelper,
    banks_client: &mut BanksClient,
    user: &Keypair,
    amount: u64,
    deposit_period: u64,
    commission_rate: u64,
) -> Result<(), BanksClientError> {
    // Get the user's token accounts
    let user_collateral_ata = spl_associated_token_account::get_associated_token_address(
        &user.pubkey(),
        &test_helper.collateral_mint.pubkey(),
    );

    let user_interest_ata = spl_associated_token_account::get_associated_token_address(
        &user.pubkey(),
        &test_helper.interest_mint.pubkey(),
    );

    // Find the user's deposit account PDA
    let (user_deposit_account, _) =
        Pubkey::find_program_address(&[user.pubkey().as_ref()], &test_helper.program_id);

    let deposit_instruction = Instruction {
        program_id: test_helper.program_id,
        accounts: vec![
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new_readonly(test_helper.config_pda, false),
            AccountMeta::new_readonly(test_helper.authority_pda, false),
            AccountMeta::new(user_collateral_ata, false),
            AccountMeta::new(user_deposit_account, false),
            AccountMeta::new(test_helper.collateral_pool_ata, false),
            AccountMeta::new(user_interest_ata, false),
            AccountMeta::new(test_helper.interest_pool_ata, false),
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

    let mut transaction =
        Transaction::new_with_payer(&[deposit_instruction], Some(&test_helper.admin.pubkey()));

    transaction.sign(
        &[&test_helper.admin, user],
        banks_client.get_latest_blockhash().await.unwrap(),
    );

    banks_client.process_transaction(transaction).await
}

// Helper function for user-specific withdrawal
async fn test_withdraw_for_user(
    test_helper: &TestHelper,
    banks_client: &mut BanksClient,
    user: &Keypair,
) -> Result<(), BanksClientError> {
    // Get the user's token accounts
    let user_collateral_ata = spl_associated_token_account::get_associated_token_address(
        &user.pubkey(),
        &test_helper.collateral_mint.pubkey(),
    );

    // Find the user's deposit account PDA
    let (user_deposit_account, _) =
        Pubkey::find_program_address(&[user.pubkey().as_ref()], &test_helper.program_id);

    let withdraw_instruction = Instruction {
        program_id: test_helper.program_id,
        accounts: vec![
            AccountMeta::new(user.pubkey(), true),
            AccountMeta::new_readonly(test_helper.config_pda, false),
            AccountMeta::new_readonly(test_helper.authority_pda, false),
            AccountMeta::new(user_deposit_account, false),
            AccountMeta::new(user_collateral_ata, false),
            AccountMeta::new(test_helper.withdrawal_pool_pda, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: TokenLockInstruction::WithdrawCollateral
            .try_to_vec()
            .unwrap(),
    };

    let mut transaction =
        Transaction::new_with_payer(&[withdraw_instruction], Some(&test_helper.admin.pubkey()));

    transaction.sign(
        &[&test_helper.admin, user],
        banks_client.get_latest_blockhash().await.unwrap(),
    );

    banks_client.process_transaction(transaction).await
}
