import {
  AddressLookupTableAccount,
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  TransactionMessage,
  VersionedTransaction,
  SendTransactionError,
} from "@solana/web3.js";
import {
  deserializeAstrapeConfig,
  deserializeUserDeposit,
  UserDepositState,
} from "../types/astrape";
import { deriveConfigPDA, deriveUserDepositPDA } from "@/utils/pda";

export class RpcClient {
  constructor(
    private connection: Connection,
    private walletPublicKey: PublicKey | null,
    private signTransaction:
      | (<T extends Transaction | VersionedTransaction>(
          transaction: T
        ) => Promise<T>)
      | undefined
  ) {}

  async signAndSendTransactionWithInstructions(
    ixs: TransactionInstruction[],
    lookupTableAccounts?: AddressLookupTableAccount[]
  ) {
    const solanaPubkey = this.walletPublicKey;
    const { signTransaction } = this;

    if (!solanaPubkey || !signTransaction)
      throw new Error("Wallet is not connected");

    const { blockhash, lastValidBlockHeight } =
      await this.connection.getLatestBlockhash();

    const msg = new TransactionMessage({
      payerKey: solanaPubkey,
      recentBlockhash: blockhash,
      instructions: ixs,
    }).compileToV0Message(lookupTableAccounts);

    const tx = new VersionedTransaction(msg);

    const signedTx = await signTransaction(tx);

    let signature: string;
    try {
      signature = await this.connection.sendRawTransaction(
        signedTx.serialize(),
        {
          preflightCommitment: "confirmed",
        }
      );
    } catch (error: unknown) {
      if (error instanceof SendTransactionError) {
        try {
          // Retrieve and output the transaction logs for easier debugging
          const logs = await error.getLogs(this.connection);
          console.error("SendTransactionError logs:", logs);
        } catch (logErr) {
          console.error("Failed to retrieve transaction logs", logErr);
        }
      }
      throw error;
    }

    await this.connection.confirmTransaction(
      {
        signature,
        lastValidBlockHeight,
        blockhash,
      },
      "confirmed"
    );

    return signature;
  }

  async getCurrentSlot() {
    return await this.connection.getSlot();
  }

  async getBlockTime(slot: number) {
    const blockTime = await this.connection.getBlockTime(slot);
    return blockTime;
  }

  async getAstrapeConfig() {
    const [configPDA] = deriveConfigPDA();

    try {
      const astrapeAccount = await this.connection.getAccountInfo(configPDA);

      if (!astrapeAccount?.data)
        throw new Error("Failed to fetch Astrape account data");

      const astrapeConfig = deserializeAstrapeConfig(astrapeAccount.data);

      return {
        interestMint: astrapeConfig.interestMint,
        collateralMint: astrapeConfig.collateralMint,
        baseInterestRate: astrapeConfig.baseInterestRate / 10, // Convert from basis points to percentage
        priceFactor: astrapeConfig.priceFactor * Math.pow(10, 8 - 6), // Adjust for decimal difference
        minCommissionRate: astrapeConfig.minCommissionRate / 10, // Convert from basis points to percentage
        maxCommissionRate: astrapeConfig.maxCommissionRate / 10, // Convert from basis points to percentage
        minDepositAmount: astrapeConfig.minDepositAmount / 10_000_000, // Convert to zBTC
        maxDepositAmount: astrapeConfig.maxDepositAmount / 10_000_000, // Convert to zBTC
        depositPeriods: astrapeConfig.depositPeriods,
      };
    } catch {
      // Fallback for testing/development
      return {
        interestMint: new PublicKey("11111111111111111111111111111111"),
        collateralMint: new PublicKey("11111111111111111111111111111111"),
        baseInterestRate: 0,
        priceFactor: 0,
        minCommissionRate: 0,
        maxCommissionRate: 0,
        minDepositAmount: 0,
        maxDepositAmount: 0,
        depositPeriods: [0],
      };
    }
  }

  async getUserDeposit() {
    if (!this.walletPublicKey) throw new Error("Wallet is not connected");

    const [userDepositPDA] = deriveUserDepositPDA(this.walletPublicKey);

    try {
      const deposits = await this.connection.getAccountInfo(userDepositPDA);

      if (!deposits?.data) throw new Error("Failed to fetch user deposit");

      const userDepositData = deserializeUserDeposit(deposits.data);
      return userDepositData;
    } catch {
      // Fallback for testing/development
      return {
        amount: 1,
        depositSlot: 366_202_735,
        unlockSlot: 382_702_735,
        interestReceived: 4048,
        state: UserDepositState.Deposited,
        commissionRate: 20,
      };
    }
  }

  async checkAccountExists(pubkey: PublicKey): Promise<boolean> {
    try {
      const accountInfo = await this.connection.getAccountInfo(pubkey);
      return accountInfo !== null;
    } catch (error) {
      console.error("Error checking if account exists:", error);
      return false;
    }
  }
}
