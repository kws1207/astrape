import useSWR from "swr";
import { useZplClient } from "@/contexts/ZplClientProvider";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { TokenLockInstruction } from "@/program/instructions";
import { useCallback } from "react";
import {
  ASTRAPE_PROGRAM_ID,
  deriveAuthorityPDA,
  deriveConfigPDA,
  deriveUserDepositPDA,
} from "@/utils/pda";

export function useAstrape() {
  const zplClient = useZplClient();
  const { data: config } = useSWR(
    zplClient ? "astrapeConfig" : null,
    async () => {
      const config = await zplClient?.getAstrapeConfig();
      return config;
    },
    {
      revalidateOnFocus: false,
      revalidateOnReconnect: false,
      revalidateIfStale: false,
    }
  );

  const { data: userDeposit, mutate: mutateUserDeposit } = useSWR(
    zplClient ? "userDeposit" : null,
    async () => {
      const deposit = await zplClient?.getUserDeposit();
      return deposit;
    }
  );

  const deposit = useCallback(
    async (amount: number, depositPeriod: number, commissionRate: number) => {
      if (!zplClient) {
        throw new Error("ZPL client not initialized");
      }

      const walletPublicKey = zplClient.getWalletPublicKey();
      if (!walletPublicKey) {
        throw new Error("Wallet not connected");
      }

      const astrapeProgramId = ASTRAPE_PROGRAM_ID;

      // Get required accounts
      const [configPDA] = deriveConfigPDA();
      const [authorityPDA] = deriveAuthorityPDA();
      const [userDepositPDA] = deriveUserDepositPDA(walletPublicKey);

      const userCollateralAccount = await zplClient.getUserCollateralAccount();
      const poolCollateralAccount = await zplClient.getPoolCollateralAccount();
      const userInterestAccount = await zplClient.getUserInterestAccount();
      const poolInterestAccount = await zplClient.getPoolInterestAccount();
      const systemProgram = new PublicKey("11111111111111111111111111111111");
      const tokenProgram = new PublicKey(
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
      );

      // Create instruction data with all required fields
      const instructionData = TokenLockInstruction.DepositCollateral({
        amount,
        deposit_period: depositPeriod,
        commission_rate: commissionRate,
      }).pack();

      // Create instruction with correct account order
      const depositInstruction = new TransactionInstruction({
        programId: astrapeProgramId,
        keys: [
          { pubkey: walletPublicKey, isSigner: true, isWritable: true },
          { pubkey: configPDA, isSigner: false, isWritable: false },
          { pubkey: authorityPDA, isSigner: false, isWritable: false },
          { pubkey: userCollateralAccount, isSigner: false, isWritable: true },
          { pubkey: userDepositPDA, isSigner: false, isWritable: true },
          { pubkey: poolCollateralAccount, isSigner: false, isWritable: true },
          { pubkey: userInterestAccount, isSigner: false, isWritable: true },
          { pubkey: poolInterestAccount, isSigner: false, isWritable: true },
          { pubkey: systemProgram, isSigner: false, isWritable: false },
          { pubkey: tokenProgram, isSigner: false, isWritable: false },
        ],
        data: instructionData,
      });

      return zplClient.signAndSendTransactionWithInstructions([
        depositInstruction,
      ]);
    },
    [zplClient]
  );

  const requestWithdrawal = useCallback(async () => {
    if (!zplClient) {
      throw new Error("ZPL client not initialized");
    }

    const walletPublicKey = zplClient.getWalletPublicKey();
    if (!walletPublicKey) {
      throw new Error("Wallet not connected");
    }

    const astrapeProgramId = ASTRAPE_PROGRAM_ID;

    // Get the user deposit PDA
    const [userDepositPDA] = deriveUserDepositPDA(walletPublicKey);

    const instructionData = TokenLockInstruction.RequestWithdrawal().pack();

    // Create instruction with correct account order (according to program)
    const requestWithdrawalInstruction = new TransactionInstruction({
      programId: astrapeProgramId,
      keys: [
        { pubkey: walletPublicKey, isSigner: true, isWritable: true },
        { pubkey: userDepositPDA, isSigner: false, isWritable: true },
      ],
      data: instructionData,
    });

    return zplClient.signAndSendTransactionWithInstructions([
      requestWithdrawalInstruction,
    ]);
  }, [zplClient]);

  const requestWithdrawalEarly = useCallback(async () => {
    if (!zplClient) {
      throw new Error("ZPL client not initialized");
    }

    const walletPublicKey = zplClient.getWalletPublicKey();
    if (!walletPublicKey) {
      throw new Error("Wallet not connected");
    }

    const astrapeProgramId = ASTRAPE_PROGRAM_ID;

    // Get required accounts
    const [configPDA] = deriveConfigPDA();
    const [authorityPDA] = deriveAuthorityPDA();
    const [userDepositPDA] = deriveUserDepositPDA(walletPublicKey);

    const userInterestAccount = await zplClient.getUserInterestAccount();
    const poolInterestAccount = await zplClient.getPoolInterestAccount();
    const tokenProgram = new PublicKey(
      "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    );

    const instructionData =
      TokenLockInstruction.RequestWithdrawalEarly().pack();

    // Create instruction with correct account order
    const requestWithdrawalEarlyInstruction = new TransactionInstruction({
      programId: astrapeProgramId,
      keys: [
        { pubkey: walletPublicKey, isSigner: true, isWritable: true },
        { pubkey: configPDA, isSigner: false, isWritable: false },
        { pubkey: authorityPDA, isSigner: false, isWritable: false },
        { pubkey: userDepositPDA, isSigner: false, isWritable: true },
        { pubkey: userInterestAccount, isSigner: false, isWritable: true },
        { pubkey: poolInterestAccount, isSigner: false, isWritable: true },
        { pubkey: tokenProgram, isSigner: false, isWritable: false },
      ],
      data: instructionData,
    });

    return zplClient.signAndSendTransactionWithInstructions([
      requestWithdrawalEarlyInstruction,
    ]);
  }, [zplClient]);

  const withdrawCollateral = useCallback(async () => {
    if (!zplClient) {
      throw new Error("ZPL client not initialized");
    }

    const walletPublicKey = zplClient.getWalletPublicKey();
    if (!walletPublicKey) {
      throw new Error("Wallet not connected");
    }

    const astrapeProgramId = ASTRAPE_PROGRAM_ID;

    // Get required accounts
    const [configPDA] = deriveConfigPDA();
    const [authorityPDA] = deriveAuthorityPDA();
    const [userDepositPDA] = deriveUserDepositPDA(walletPublicKey);

    const userCollateralAccount = await zplClient.getUserCollateralAccount();
    const withdrawalPoolAccount = await zplClient.getWithdrawalPoolAccount();
    const tokenProgram = new PublicKey(
      "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    );

    const withdrawCollateralInstruction = new TransactionInstruction({
      programId: astrapeProgramId,
      keys: [
        { pubkey: walletPublicKey, isSigner: true, isWritable: true },
        { pubkey: configPDA, isSigner: false, isWritable: false },
        { pubkey: authorityPDA, isSigner: false, isWritable: false },
        { pubkey: userDepositPDA, isSigner: false, isWritable: true },
        { pubkey: userCollateralAccount, isSigner: false, isWritable: true },
        { pubkey: withdrawalPoolAccount, isSigner: false, isWritable: true },
        { pubkey: tokenProgram, isSigner: false, isWritable: false },
      ],
      data: TokenLockInstruction.WithdrawCollateral().pack(),
    });

    return zplClient.signAndSendTransactionWithInstructions([
      withdrawCollateralInstruction,
    ]);
  }, [zplClient]);

  return {
    config,
    deposit,
    requestWithdrawal,
    requestWithdrawalEarly,
    withdrawCollateral,
    userDeposit,
    mutateUserDeposit,
  };
}
