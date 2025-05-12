import { PublicKey } from "@solana/web3.js";

// Program ID
export const ASTRAPE_PROGRAM_ID = new PublicKey(
  "5oDdrYxYbeABKyNyZHsgsJBREZjwZurzHcRPNGxtYPXn"
);

// PDA seed constants from processor.rs
export const CONFIG_SEED = Buffer.from("pool_config");
export const AUTHORITY_SEED = Buffer.from("authority");
export const WITHDRAWAL_POOL_SEED = Buffer.from("withdrawal_pool");
export const COLLATERAL_POOL_SEED = Buffer.from("collateral_pool");
export const INTEREST_POOL_SEED = Buffer.from("interest_pool");

/**
 * Derive the configuration PDA address
 */
export function deriveConfigPDA(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([CONFIG_SEED], ASTRAPE_PROGRAM_ID);
}

/**
 * Derive the authority PDA address
 */
export function deriveAuthorityPDA(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([AUTHORITY_SEED], ASTRAPE_PROGRAM_ID);
}

/**
 * Derive the withdrawal pool PDA address
 */
export function deriveWithdrawalPoolPDA(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [WITHDRAWAL_POOL_SEED],
    ASTRAPE_PROGRAM_ID
  );
}

/**
 * Derive the user deposit PDA address
 * @param userPublicKey User's public key
 */
export function deriveUserDepositPDA(
  userPublicKey: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [userPublicKey.toBuffer()],
    ASTRAPE_PROGRAM_ID
  );
}

/**
 * Derive the collateral pool PDA address
 * @param collateralMint Collateral mint public key
 */
export function deriveCollateralPoolPDA(
  collateralMint: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("collateral_pool"), collateralMint.toBuffer()],
    ASTRAPE_PROGRAM_ID
  );
}

/**
 * Derive the interest pool PDA address
 * @param interestMint Interest mint public key
 */
export function deriveInterestPoolPDA(
  interestMint: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("interest_pool"), interestMint.toBuffer()],
    ASTRAPE_PROGRAM_ID
  );
}
