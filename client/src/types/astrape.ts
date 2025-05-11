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

export enum UserDepositState {
  Deposited = 0,
  WithdrawRequested = 1,
  WithdrawReady = 2,
}

export interface UserDeposit {
  amount: number;
  depositSlot: number;
  unlockSlot: number;
  interestReceived: number;
  state: UserDepositState;
  commissionRate: number;
}

export const userDepositStateSchema = borsh.rustEnum([
  borsh.struct([], "Deposited"),
  borsh.struct([], "WithdrawRequested"),
  borsh.struct([], "WithdrawReady"),
]);

export const userDepositSchema: Structure<UserDeposit> = borsh.struct([
  borsh.u64("amount"),
  borsh.u64("depositSlot"),
  borsh.u64("unlockSlot"),
  borsh.u64("interestReceived"),
  borsh.u8("state"),
  borsh.u64("commissionRate"),
]);

export interface PoolState {
  deposits: Map<string, UserDeposit>;
}

export const poolStateSchema: Structure<PoolState> = borsh.struct([
  borsh.map(borsh.publicKey(), userDepositSchema, "deposits"),
]);

export function deserializePoolState(data: Buffer): PoolState {
  if (!data) throw new Error("Data is undefined");
  return poolStateSchema.decode(data);
}
