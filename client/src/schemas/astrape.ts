import * as borsh from "@coral-xyz/borsh";
import { Structure } from "@solana/buffer-layout";
import BN from "bn.js";

import {
  AstrapeConfig,
  UserDeposit,
  UserDepositState,
  PoolState,
} from "@/types/astrape";

/* -------------------------------------------------------------------------- */
/*                               Astrape Config                               */
/* -------------------------------------------------------------------------- */

export const astrapeConfigSchema: Structure<AstrapeConfig> = borsh.struct([
  borsh.publicKey("interestMint"),
  borsh.publicKey("collateralMint"),
  borsh.u64("baseInterestRate"),
  borsh.u64("priceFactor"),
  borsh.u64("minCommissionRate"),
  borsh.u64("maxCommissionRate"),
  borsh.u64("minDepositAmount"),
  borsh.u64("maxDepositAmount"),
  borsh.vec(borsh.u64(), "depositPeriods"),
]);

export function deserializeAstrapeConfig(data: Buffer): AstrapeConfig {
  if (!data) throw new Error("Data is undefined");
  const decoded = astrapeConfigSchema.decode(data);
  return {
    ...decoded,
    minDepositAmount: decoded.minDepositAmount / 10,
    maxDepositAmount: decoded.maxDepositAmount / 10,
  };
}

/* -------------------------------------------------------------------------- */
/*                               User Deposit                                 */
/* -------------------------------------------------------------------------- */

export const userDepositStateSchema = borsh.rustEnum([
  borsh.struct([], "Deposited"),
  borsh.struct([], "WithdrawRequested"),
  borsh.struct([], "WithdrawReady"),
  borsh.struct([], "WithdrawCompleted"),
]);

export const userDepositSchema: Structure<UserDeposit> = borsh.struct([
  borsh.u64("amount"),
  borsh.u64("depositSlot"),
  borsh.u64("unlockSlot"),
  borsh.u64("interestReceived"),
  borsh.u8("state"),
  borsh.u64("commissionRate"),
]);

export function deserializeUserDeposit(data: Buffer): UserDeposit {
  if (!data) throw new Error("Data is undefined");
  const decoded = userDepositSchema.decode(data);

  const COLLATERAL_DECIMALS = 8; // zBTC has 8 decimals (satoshis)
  const INTEREST_DECIMALS = 6; // USDC has 6 decimals
  const COMMISSION_DECIMALS = 1; // Commission stored in tenths of a percent

  return {
    amount:
      (decoded.amount as unknown as BN).toNumber() /
      Math.pow(10, COLLATERAL_DECIMALS),
    depositSlot: (decoded.depositSlot as unknown as BN).toNumber(),
    unlockSlot: (decoded.unlockSlot as unknown as BN).toNumber(),
    interestReceived:
      (decoded.interestReceived as unknown as BN).toNumber() /
      Math.pow(10, INTEREST_DECIMALS),
    state: decoded.state as UserDepositState,
    commissionRate:
      (decoded.commissionRate as unknown as BN).toNumber() /
      Math.pow(10, COMMISSION_DECIMALS),
  };
}

/* -------------------------------------------------------------------------- */
/*                                  Pool                                      */
/* -------------------------------------------------------------------------- */

export const poolStateSchema: Structure<PoolState> = borsh.struct([
  borsh.map(borsh.publicKey(), userDepositSchema, "deposits"),
]);

export function deserializePoolState(data: Buffer): PoolState {
  if (!data) throw new Error("Data is undefined");
  return poolStateSchema.decode(data);
}
