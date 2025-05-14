import { PublicKey } from "@solana/web3.js";

export interface AstrapeConfig {
  interestMint: PublicKey;
  collateralMint: PublicKey;
  baseInterestRate: number;
  priceFactor: number;
  minCommissionRate: number;
  maxCommissionRate: number;
  minDepositAmount: number;
  maxDepositAmount: number;
  depositPeriods: number[];
}

export enum UserDepositState {
  Deposited = 0,
  WithdrawRequested = 1,
  WithdrawReady = 2,
  WithdrawCompleted = 3,
}

export interface UserDeposit {
  amount: number;
  depositSlot: number;
  unlockSlot: number;
  interestReceived: number;
  state: UserDepositState;
  commissionRate: number;
}

export interface PoolState {
  deposits: Map<string, UserDeposit>;
}
