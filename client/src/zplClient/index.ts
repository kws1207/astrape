import {
  AddressLookupTableAccount,
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  VersionedTransaction,
} from "@solana/web3.js";
import BN from "bn.js";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";

import { BitcoinAddress, BitcoinXOnlyPublicKey } from "@/types/wallet";

import { AccountService } from "./account";
import { InstructionService } from "./instruction";
import { RpcClient } from "./rpcClient";

export class ZplClient {
  private accountService: AccountService;
  private instructionService: InstructionService;
  private rpcClient: RpcClient;
  private twoWayPegProgramId: PublicKey;
  private liquidityManagementProgramId: PublicKey;
  private assetMint: PublicKey;
  public readonly astrapeProgramId: PublicKey;

  constructor(
    connection: Connection,
    private walletPublicKey: PublicKey | null,
    signTransaction:
      | (<T extends Transaction | VersionedTransaction>(
          transaction: T
        ) => Promise<T>)
      | undefined,
    twoWayPegProgramId: string,
    liquidityManagementProgramId: string,
    assetMint: string
  ) {
    this.twoWayPegProgramId = new PublicKey(twoWayPegProgramId);
    this.liquidityManagementProgramId = new PublicKey(
      liquidityManagementProgramId
    );
    this.assetMint = new PublicKey(assetMint);

    // Set Astrape program ID
    if (process.env.NEXT_PUBLIC_ASTRAPE_PROGRAM_CONFIG_ACCOUNT_ADDRESS_BASE58) {
      this.astrapeProgramId = new PublicKey(
        process.env.NEXT_PUBLIC_ASTRAPE_PROGRAM_CONFIG_ACCOUNT_ADDRESS_BASE58
      );
    } else {
      // throw new Error("Astrape program address is not set");
      this.astrapeProgramId = new PublicKey("11111111111111111111111111111111");
    }

    this.accountService = new AccountService(
      connection,
      this.twoWayPegProgramId,
      this.liquidityManagementProgramId
    );

    this.instructionService = new InstructionService(
      walletPublicKey,
      this.twoWayPegProgramId,
      this.liquidityManagementProgramId,
      this.assetMint,
      this.accountService
    );

    this.rpcClient = new RpcClient(
      connection,
      walletPublicKey,
      signTransaction
    );
  }

  // Account service methods
  deriveConfigurationAddress() {
    return this.accountService.deriveConfigurationAddress();
  }

  deriveCpiIdentityAddress() {
    return this.accountService.deriveCpiIdentityAddress();
  }

  deriveHotReserveBucketAddress(
    hotReserveBitcoinXOnlyPublicKey: BitcoinXOnlyPublicKey
  ): PublicKey {
    return this.accountService.deriveHotReserveBucketAddress(
      hotReserveBitcoinXOnlyPublicKey
    );
  }

  deriveInteraction(seed1: Buffer, seed2: BN) {
    return this.accountService.deriveInteraction(seed1, seed2);
  }

  deriveLiquidityManagementConfigurationAddress() {
    return this.accountService.deriveLiquidityManagementConfigurationAddress();
  }

  deriveLiquidityManagementGuardianSettingAddress(
    twoWayPegGuardianSetting: PublicKey
  ) {
    return this.accountService.deriveLiquidityManagementGuardianSettingAddress(
      twoWayPegGuardianSetting
    );
  }

  deriveSplTokenVaultAuthorityAddress(twoWayPegGuardianSetting: PublicKey) {
    return this.accountService.deriveSplTokenVaultAuthorityAddress(
      twoWayPegGuardianSetting
    );
  }

  derivePositionAddress(
    lmGuardianSetting: PublicKey,
    userAddress: PublicKey | null
  ): PublicKey {
    return this.accountService.derivePositionAddress(
      lmGuardianSetting,
      userAddress
    );
  }

  async getTwoWayPegConfiguration() {
    return this.accountService.getTwoWayPegConfiguration();
  }

  async getColdReserveBuckets() {
    return this.accountService.getColdReserveBuckets();
  }

  async getHotReserveBucketsByBitcoinXOnlyPubkey(
    bitcoinXOnlyPubkey: BitcoinXOnlyPublicKey
  ) {
    return this.accountService.getHotReserveBucketsByBitcoinXOnlyPubkey(
      bitcoinXOnlyPubkey
    );
  }

  async getHotReserveBucketsBySolanaPubkey(solanaPubkey: PublicKey) {
    return this.accountService.getHotReserveBucketsBySolanaPubkey(solanaPubkey);
  }

  async getPositionsByWallet(solanaPubkey: PublicKey) {
    return this.accountService.getPositionsByWallet(solanaPubkey);
  }

  // Instruction service methods
  constructCreateHotReserveBucketIx(
    solanaPubkey: PublicKey,
    hotReserveBitcoinXOnlyPublicKey: BitcoinXOnlyPublicKey,
    userBitcoinXOnlyPublicKey: BitcoinXOnlyPublicKey,
    unlockBlockHeight: number,
    guardianSetting: PublicKey,
    guardianCertificate: PublicKey,
    coldReserveBucket: PublicKey,
    layerFeeCollector: PublicKey
  ) {
    return this.instructionService.constructCreateHotReserveBucketIx(
      solanaPubkey,
      hotReserveBitcoinXOnlyPublicKey,
      userBitcoinXOnlyPublicKey,
      unlockBlockHeight,
      guardianSetting,
      guardianCertificate,
      coldReserveBucket,
      layerFeeCollector
    );
  }

  constructReactivateHotReserveBucketIx(
    hotReserveBucketPda: PublicKey,
    layerFeeCollector: PublicKey
  ) {
    return this.instructionService.constructReactivateHotReserveBucketIx(
      hotReserveBucketPda,
      layerFeeCollector
    );
  }

  constructAddWithdrawalRequestIx(
    solanaPubkey: PublicKey,
    amount: BN,
    receiverAddress: BitcoinAddress,
    guardianSetting: PublicKey,
    layerFeeCollector: PublicKey
  ) {
    return this.instructionService.constructAddWithdrawalRequestIx(
      solanaPubkey,
      amount,
      receiverAddress,
      guardianSetting,
      layerFeeCollector
    );
  }

  constructRetrieveIx(
    amount: BN,
    guardianSetting: PublicKey,
    receiverAta: PublicKey
  ) {
    return this.instructionService.constructRetrieveIx(
      amount,
      guardianSetting,
      receiverAta
    );
  }

  constructStoreIx(amount: BN, guardianSetting: PublicKey) {
    return this.instructionService.constructStoreIx(amount, guardianSetting);
  }

  // RPC client methods
  async signAndSendTransactionWithInstructions(
    ixs: TransactionInstruction[],
    lookupTableAccounts?: AddressLookupTableAccount[]
  ) {
    return this.rpcClient.signAndSendTransactionWithInstructions(
      ixs,
      lookupTableAccounts
    );
  }

  async getPoolConfig() {
    return this.rpcClient.getPoolConfig();
  }

  async getUserDeposit() {
    return this.rpcClient.getUserDeposit();
  }

  // Added methods for Astrape functionality

  getWalletPublicKey(): PublicKey | null {
    return this.walletPublicKey;
  }

  async getCurrentSlot() {
    return this.rpcClient.getCurrentSlot();
  }

  async getBlockTime(slot: number) {
    for (let i = 0; i < 3; i++) {
      try {
        return await this.rpcClient.getBlockTime(slot + i);
      } catch {
        if (i === 2) {
          return null;
        }
      }
    }
  }

  async getPoolStateAccount(): Promise<PublicKey> {
    const [stateAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool_state")],
      this.astrapeProgramId
    );
    return stateAccount;
  }

  async getUserCollateralAccount(): Promise<PublicKey> {
    if (!this.walletPublicKey) {
      throw new Error("Wallet not connected");
    }

    const poolConfig = await this.getPoolConfig();
    return getAssociatedTokenAddressSync(
      poolConfig.collateralMint,
      this.walletPublicKey
    );
  }

  async getPoolCollateralAccount(): Promise<PublicKey> {
    const poolConfig = await this.getPoolConfig();
    const [collateralPool] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateral_pool"), poolConfig.collateralMint.toBuffer()],
      this.astrapeProgramId
    );
    return collateralPool;
  }

  async getUserInterestAccount(): Promise<PublicKey> {
    if (!this.walletPublicKey) {
      throw new Error("Wallet not connected");
    }

    const poolConfig = await this.getPoolConfig();
    return getAssociatedTokenAddressSync(
      poolConfig.interestMint,
      this.walletPublicKey
    );
  }

  async getPoolInterestAccount(): Promise<PublicKey> {
    const poolConfig = await this.getPoolConfig();
    const [interestPool] = PublicKey.findProgramAddressSync(
      [Buffer.from("interest_pool"), poolConfig.interestMint.toBuffer()],
      this.astrapeProgramId
    );
    return interestPool;
  }
}
