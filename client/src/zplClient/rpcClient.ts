import {
  AddressLookupTableAccount,
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";
import { deserializePoolConfig } from "../types/astrape";

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

    const signature = await this.connection.sendRawTransaction(
      signedTx.serialize(),
      {
        preflightCommitment: "confirmed",
      }
    );

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

  async getPoolConfig() {
    if (!process.env.NEXT_PUBLIC_ASTRAPE_PROGRAM_ADDRESS_BASE58) {
      // throw new Error("Astrape program address is not set");
      return {
        admin: new PublicKey("11111111111111111111111111111111"),
        interestMint: new PublicKey("11111111111111111111111111111111"),
        collateralMint: new PublicKey("11111111111111111111111111111111"),
        priceFactor: 0,
        baseInterestRate: 0,
        minCommissionRate: 0,
        maxCommissionRate: 0,
        minDepositAmount: 0,
        maxDepositAmount: 0,
        depositPeriod: [0],
      };
    }

    const astrapeAccount = await this.connection.getAccountInfo(
      new PublicKey(process.env.NEXT_PUBLIC_ASTRAPE_PROGRAM_ADDRESS_BASE58)
    );

    if (!astrapeAccount?.data)
      throw new Error("Failed to fetch Astrape account data");

    const poolConfig = deserializePoolConfig(astrapeAccount.data);

    return poolConfig;
  }
}
