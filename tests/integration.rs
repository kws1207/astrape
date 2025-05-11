let initialize_instruction = Instruction {
    program_id: self.program_id,
    accounts: vec![
        AccountMeta::new(self.admin.pubkey(), true),                      // Admin (payer & signer)
        AccountMeta::new(self.config_pda, false),                         // Config PDA
        AccountMeta::new(self.state_pda, false),                          // State PDA
        AccountMeta::new_readonly(solana_program::system_program::id(), false), // System program
        AccountMeta::new_readonly(spl_token::id(), false),                // Token program
        AccountMeta::new_readonly(spl_associated_token_account::id(), false), // ATA program
        AccountMeta::new(self.interest_pool_ata, false),                  // Interest pool ATA
        AccountMeta::new(self.collateral_pool_ata, false),                // Collateral pool ATA
        AccountMeta::new_readonly(self.interest_mint.pubkey(), false),    // Interest mint
        AccountMeta::new_readonly(self.collateral_mint.pubkey(), false),  // Collateral mint
        AccountMeta::new_readonly(self.program_id, false),                // Program account (for authority)
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