import * as borsh from "@coral-xyz/borsh";
import { Structure } from "@solana/buffer-layout";
import { PublicKey } from "@solana/web3.js";

export interface PoolConfig {
  admin: PublicKey;
  interestMint: PublicKey;
  collateralMint: PublicKey;
  priceFactor: number;
  baseInterestRate: number;
  minCommissionRate: number;
  maxCommissionRate: number;
  minDepositAmount: number;
  maxDepositAmount: number;
  depositPeriod: number[];
}

export const poolConfigSchema: Structure<PoolConfig> = borsh.struct([
  borsh.publicKey("admin"),
  borsh.publicKey("interestMint"),
  borsh.publicKey("collateralMint"),
  borsh.u64("priceFactor"),
  borsh.u64("baseInterestRate"),
  borsh.u64("minCommissionRate"),
  borsh.u64("maxCommissionRate"),
  borsh.u64("minDepositAmount"),
  borsh.u64("maxDepositAmount"),
  borsh.vec(borsh.u64(), "depositPeriod"),
]);

export function deserializePoolConfig(data: Buffer): PoolConfig {
  if (!data) throw new Error("Data is undefined");
  return poolConfigSchema.decode(data);
}
