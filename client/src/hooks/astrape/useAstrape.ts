import useSWR from "swr";
import { useZplClient } from "@/contexts/ZplClientProvider";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { TokenLockInstruction } from "@/program/instructions";
import { useCallback } from "react";

export function useAstrape() {
  const zplClient = useZplClient();
  const { data: poolConfig } = useSWR(
    zplClient ? "poolConfig" : null,
    async () => {
      const config = await zplClient?.getPoolConfig();
      return config;
    },
    {
      revalidateOnFocus: false,
      revalidateOnReconnect: false,
      revalidateIfStale: false,
    }
  );

  const deposit = useCallback(
    async (amount: number, unlockSlot: number) => {
      if (!zplClient) {
        throw new Error("ZPL client not initialized");
      }

      const walletPublicKey = zplClient.getWalletPublicKey();
      if (!walletPublicKey) {
        throw new Error("Wallet not connected");
      }

      const astrapeProgramId = new PublicKey(
        process.env.NEXT_PUBLIC_ASTRAPE_PROGRAM_ADDRESS_BASE58 || ""
      );

      const poolStateAccount = await zplClient.getPoolStateAccount();

      const userCollateralAccount = await zplClient.getUserCollateralAccount();
      const poolCollateralAccount = await zplClient.getPoolCollateralAccount();
      const userInterestAccount = await zplClient.getUserInterestAccount();
      const poolInterestAccount = await zplClient.getPoolInterestAccount();

      const depositInstruction = new TransactionInstruction({
        programId: astrapeProgramId,
        keys: [
          { pubkey: poolStateAccount, isSigner: false, isWritable: true },
          { pubkey: userCollateralAccount, isSigner: false, isWritable: true },
          { pubkey: poolCollateralAccount, isSigner: false, isWritable: true },
          { pubkey: userInterestAccount, isSigner: false, isWritable: true },
          { pubkey: poolInterestAccount, isSigner: false, isWritable: true },
        ],
        data: TokenLockInstruction.DepositCollateral({
          unlock_slot: unlockSlot,
        }).pack(),
      });

      return zplClient.signAndSendTransactionWithInstructions([
        depositInstruction,
      ]);
    },
    [zplClient]
  );

  return { poolConfig, deposit };
}
