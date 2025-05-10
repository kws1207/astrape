import { PublicKey } from "@solana/web3.js";

export class TokenLockInstruction {
  static AdminCreateConfig({
    interest_mint,
    collateral_mint,
    base_interest_rate,
    price_factor,
    min_commission_rate,
    max_commission_rate,
    min_deposit_amount,
    max_deposit_amount,
    deposit_periods,
  }: {
    interest_mint: PublicKey;
    collateral_mint: PublicKey;
    base_interest_rate: number;
    price_factor: number;
    min_commission_rate: number;
    max_commission_rate: number;
    min_deposit_amount: number;
    max_deposit_amount: number;
    deposit_periods: number[];
  }): TokenLockInstruction {
    return new TokenLockInstruction("AdminCreateConfig", {
      interest_mint,
      collateral_mint,
      base_interest_rate,
      price_factor,
      min_commission_rate,
      max_commission_rate,
      min_deposit_amount,
      max_deposit_amount,
      deposit_periods,
    });
  }

  static AdminUpdateConfig({
    param,
    base_interest_rate,
    price_factor,
  }: {
    param: number;
    base_interest_rate?: number;
    price_factor?: number;
  }): TokenLockInstruction {
    return new TokenLockInstruction("AdminUpdateConfig", {
      param,
      base_interest_rate,
      price_factor,
    });
  }

  static AdminWithdrawCollateralForInvestment(): TokenLockInstruction {
    return new TokenLockInstruction("AdminWithdrawCollateralForInvestment", {});
  }

  static AdminUpdateDepositStates(): TokenLockInstruction {
    return new TokenLockInstruction("AdminUpdateDepositStates", {});
  }

  static AdminPrepareWithdrawal({
    user_pubkey,
  }: {
    user_pubkey: PublicKey;
  }): TokenLockInstruction {
    return new TokenLockInstruction("AdminPrepareWithdrawal", { user_pubkey });
  }

  static AdminDepositInterest({
    amount,
  }: {
    amount: number;
  }): TokenLockInstruction {
    return new TokenLockInstruction("AdminDepositInterest", { amount });
  }

  static AdminWithdrawInterest({
    amount,
  }: {
    amount: number;
  }): TokenLockInstruction {
    return new TokenLockInstruction("AdminWithdrawInterest", { amount });
  }

  static DepositCollateral({
    unlock_slot,
  }: {
    unlock_slot: number;
  }): TokenLockInstruction {
    return new TokenLockInstruction("DepositCollateral", { unlock_slot });
  }

  static RequestWithdrawal(): TokenLockInstruction {
    return new TokenLockInstruction("RequestWithdrawal", {});
  }

  static WithdrawCollateral(): TokenLockInstruction {
    return new TokenLockInstruction("WithdrawCollateral", {});
  }

  constructor(
    public readonly type: string,
    public readonly params: Record<string, unknown>
  ) {}

  private numberToLEBytes(num: number): Uint8Array {
    return new Uint8Array(new BigUint64Array([BigInt(num)]).buffer);
  }

  pack(): Buffer {
    const buffer: number[] = [];

    switch (this.type) {
      case "AdminCreateConfig": {
        const {
          interest_mint,
          collateral_mint,
          base_interest_rate,
          price_factor,
          min_commission_rate,
          max_commission_rate,
          min_deposit_amount,
          max_deposit_amount,
          deposit_periods,
        } = this.params as {
          interest_mint: PublicKey;
          collateral_mint: PublicKey;
          base_interest_rate: number;
          price_factor: number;
          min_commission_rate: number;
          max_commission_rate: number;
          min_deposit_amount: number;
          max_deposit_amount: number;
          deposit_periods: number[];
        };

        buffer.push(0);

        const interestMintBytes = interest_mint.toBytes();
        for (let i = 0; i < interestMintBytes.length; i++) {
          buffer.push(interestMintBytes[i]);
        }

        const collateralMintBytes = collateral_mint.toBytes();
        for (let i = 0; i < collateralMintBytes.length; i++) {
          buffer.push(collateralMintBytes[i]);
        }

        const baseInterestBytes = this.numberToLEBytes(base_interest_rate);
        const priceFactorBytes = this.numberToLEBytes(price_factor);
        const minCommissionBytes = this.numberToLEBytes(min_commission_rate);
        const maxCommissionBytes = this.numberToLEBytes(max_commission_rate);
        const minDepositBytes = this.numberToLEBytes(min_deposit_amount);
        const maxDepositBytes = this.numberToLEBytes(max_deposit_amount);

        for (let i = 0; i < baseInterestBytes.length; i++)
          buffer.push(baseInterestBytes[i]);
        for (let i = 0; i < priceFactorBytes.length; i++)
          buffer.push(priceFactorBytes[i]);
        for (let i = 0; i < minCommissionBytes.length; i++)
          buffer.push(minCommissionBytes[i]);
        for (let i = 0; i < maxCommissionBytes.length; i++)
          buffer.push(maxCommissionBytes[i]);
        for (let i = 0; i < minDepositBytes.length; i++)
          buffer.push(minDepositBytes[i]);
        for (let i = 0; i < maxDepositBytes.length; i++)
          buffer.push(maxDepositBytes[i]);

        // Encode deposit periods
        for (const period of deposit_periods) {
          const periodBytes = this.numberToLEBytes(period);
          for (let i = 0; i < periodBytes.length; i++)
            buffer.push(periodBytes[i]);
        }
        break;
      }
      case "AdminUpdateConfig": {
        const { param, base_interest_rate, price_factor } = this.params as {
          param: number;
          base_interest_rate?: number;
          price_factor?: number;
        };

        buffer.push(1);
        buffer.push(param);

        if (base_interest_rate !== undefined) {
          buffer.push(1);
          const rateBytes = this.numberToLEBytes(base_interest_rate);
          for (let i = 0; i < rateBytes.length; i++) buffer.push(rateBytes[i]);
        } else {
          buffer.push(0);
        }

        if (price_factor !== undefined) {
          buffer.push(1);
          const factorBytes = this.numberToLEBytes(price_factor);
          for (let i = 0; i < factorBytes.length; i++)
            buffer.push(factorBytes[i]);
        } else {
          buffer.push(0);
        }
        break;
      }
      case "AdminWithdrawCollateralForInvestment": {
        buffer.push(2);
        break;
      }
      case "AdminUpdateDepositStates": {
        buffer.push(3);
        break;
      }
      case "AdminPrepareWithdrawal": {
        const { user_pubkey } = this.params as { user_pubkey: PublicKey };
        buffer.push(4);

        const userPubkeyBytes = user_pubkey.toBytes();
        for (let i = 0; i < userPubkeyBytes.length; i++) {
          buffer.push(userPubkeyBytes[i]);
        }

        break;
      }
      case "AdminDepositInterest": {
        const { amount } = this.params as { amount: number };
        buffer.push(5);
        const amountBytes = this.numberToLEBytes(amount);
        for (let i = 0; i < amountBytes.length; i++)
          buffer.push(amountBytes[i]);
        break;
      }
      case "AdminWithdrawInterest": {
        const { amount } = this.params as { amount: number };
        buffer.push(6);
        const amountBytes = this.numberToLEBytes(amount);
        for (let i = 0; i < amountBytes.length; i++)
          buffer.push(amountBytes[i]);
        break;
      }
      case "DepositCollateral": {
        const { unlock_slot } = this.params as { unlock_slot: number };
        buffer.push(7);
        const slotBytes = this.numberToLEBytes(unlock_slot);
        for (let i = 0; i < slotBytes.length; i++) buffer.push(slotBytes[i]);
        break;
      }
      case "RequestWithdrawal": {
        buffer.push(8);
        break;
      }
      case "WithdrawCollateral": {
        buffer.push(9);
        break;
      }
    }

    return Buffer.from(buffer);
  }
}
