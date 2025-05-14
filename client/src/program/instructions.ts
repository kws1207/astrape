import { PublicKey } from "@solana/web3.js";

export class TokenLockInstruction {
  static Initialize({
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
    return new TokenLockInstruction("Initialize", {
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
    min_commission_rate,
    max_commission_rate,
    min_deposit_amount,
    max_deposit_amount,
    deposit_periods,
  }: {
    param: number;
    base_interest_rate?: number;
    price_factor?: number;
    min_commission_rate?: number;
    max_commission_rate?: number;
    min_deposit_amount?: number;
    max_deposit_amount?: number;
    deposit_periods?: number[];
  }): TokenLockInstruction {
    return new TokenLockInstruction("AdminUpdateConfig", {
      param,
      base_interest_rate,
      price_factor,
      min_commission_rate,
      max_commission_rate,
      min_deposit_amount,
      max_deposit_amount,
      deposit_periods,
    });
  }

  static AdminWithdrawCollateralForInvestment(): TokenLockInstruction {
    return new TokenLockInstruction("AdminWithdrawCollateralForInvestment", {});
  }

  static AdminPrepareWithdrawal(): TokenLockInstruction {
    return new TokenLockInstruction("AdminPrepareWithdrawal", {});
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
    amount,
    deposit_period,
    commission_rate,
  }: {
    amount: number;
    deposit_period: number;
    commission_rate: number;
  }): TokenLockInstruction {
    return new TokenLockInstruction("DepositCollateral", {
      amount,
      deposit_period,
      commission_rate,
    });
  }

  static RequestWithdrawalEarly(): TokenLockInstruction {
    return new TokenLockInstruction("RequestWithdrawalEarly", {});
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

  private numberToLEBytes(num: number, decimals: number = 0): Uint8Array {
    const intValue = Math.round(num * Math.pow(10, decimals));
    return new Uint8Array(new BigUint64Array([BigInt(intValue)]).buffer);
  }

  pack(): Buffer {
    const buffer: number[] = [];

    switch (this.type) {
      case "Initialize": {
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

        for (const period of deposit_periods) {
          const periodBytes = this.numberToLEBytes(period);
          for (let i = 0; i < periodBytes.length; i++)
            buffer.push(periodBytes[i]);
        }
        break;
      }
      case "AdminUpdateConfig": {
        const {
          param,
          base_interest_rate,
          price_factor,
          min_commission_rate,
          max_commission_rate,
          min_deposit_amount,
          max_deposit_amount,
          deposit_periods,
        } = this.params as {
          param: number;
          base_interest_rate?: number;
          price_factor?: number;
          min_commission_rate?: number;
          max_commission_rate?: number;
          min_deposit_amount?: number;
          max_deposit_amount?: number;
          deposit_periods?: number[];
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

        if (min_commission_rate !== undefined) {
          buffer.push(1);
          const rateBytes = this.numberToLEBytes(min_commission_rate);
          for (let i = 0; i < rateBytes.length; i++) buffer.push(rateBytes[i]);
        } else {
          buffer.push(0);
        }

        if (max_commission_rate !== undefined) {
          buffer.push(1);
          const rateBytes = this.numberToLEBytes(max_commission_rate);
          for (let i = 0; i < rateBytes.length; i++) buffer.push(rateBytes[i]);
        } else {
          buffer.push(0);
        }

        if (min_deposit_amount !== undefined) {
          buffer.push(1);
          const amountBytes = this.numberToLEBytes(min_deposit_amount);
          for (let i = 0; i < amountBytes.length; i++)
            buffer.push(amountBytes[i]);
        } else {
          buffer.push(0);
        }

        if (max_deposit_amount !== undefined) {
          buffer.push(1);
          const amountBytes = this.numberToLEBytes(max_deposit_amount);
          for (let i = 0; i < amountBytes.length; i++)
            buffer.push(amountBytes[i]);
        } else {
          buffer.push(0);
        }

        if (deposit_periods !== undefined) {
          buffer.push(1);
          for (const period of deposit_periods) {
            const periodBytes = this.numberToLEBytes(period);
            for (let i = 0; i < periodBytes.length; i++)
              buffer.push(periodBytes[i]);
          }
        } else {
          buffer.push(0);
        }

        break;
      }
      case "AdminWithdrawCollateralForInvestment": {
        buffer.push(2);
        break;
      }
      case "AdminPrepareWithdrawal": {
        buffer.push(3);
        break;
      }
      case "AdminDepositInterest": {
        const { amount } = this.params as { amount: number };
        buffer.push(4);
        const amountBytes = this.numberToLEBytes(amount, 6);
        for (let i = 0; i < amountBytes.length; i++)
          buffer.push(amountBytes[i]);
        break;
      }
      case "AdminWithdrawInterest": {
        const { amount } = this.params as { amount: number };
        buffer.push(5);
        const amountBytes = this.numberToLEBytes(amount, 6);
        for (let i = 0; i < amountBytes.length; i++)
          buffer.push(amountBytes[i]);
        break;
      }
      case "DepositCollateral": {
        const { amount, deposit_period, commission_rate } = this.params as {
          amount: number;
          deposit_period: number;
          commission_rate: number;
        };
        buffer.push(6);

        const amountBytes = this.numberToLEBytes(amount, 8);
        for (let i = 0; i < amountBytes.length; i++)
          buffer.push(amountBytes[i]);

        const periodBytes = this.numberToLEBytes(deposit_period);
        for (let i = 0; i < periodBytes.length; i++)
          buffer.push(periodBytes[i]);

        const commissionBytes = this.numberToLEBytes(commission_rate, 1);
        for (let i = 0; i < commissionBytes.length; i++)
          buffer.push(commissionBytes[i]);
        break;
      }
      case "RequestWithdrawalEarly": {
        buffer.push(7);
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
