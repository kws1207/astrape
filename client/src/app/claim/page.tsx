"use client";

import { motion } from "framer-motion";

import Icon from "@/components/Icons";
import { useMemo, useState } from "react";
import { useAstrape } from "@/hooks/astrape/useAstrape";

type DepositPeriod = "1M" | "3M" | "6M";

const depositPeriodsDisplayText: Record<DepositPeriod, string> = {
  "1M": "1 Month",
  "3M": "3 Months",
  "6M": "6 Months",
};

const slotCountMap: Record<DepositPeriod, number> = {
  "1M": 1 * 30 * 24 * 60 * 60 * 2.5,
  "3M": 3 * 30 * 24 * 60 * 60 * 2.5,
  "6M": 6 * 30 * 24 * 60 * 60 * 2.5,
};

export default function ClaimPage() {
  const [step, setStep] = useState<"amount-and-period" | "commission">(
    "amount-and-period"
  );
  const [depositAmount, setDepositAmount] = useState(1);
  const [depositPeriod, setDepositPeriod] = useState<DepositPeriod>("3M");
  const [commissionRate, setCommissionRate] = useState(20);

  const astrape = useAstrape();

  const onChangeDepositAmount = (
    event: React.ChangeEvent<HTMLInputElement>
  ) => {
    setDepositAmount(Math.round(event.target.valueAsNumber * 100) / 100);
  };

  const onChangeCommissionRate = (
    event: React.ChangeEvent<HTMLInputElement>
  ) => {
    const newRate = Math.round(event.target.valueAsNumber * 100) / 100;
    setCommissionRate(Math.max(20, Math.min(40, newRate)));
  };

  const onClickDeposit = () => {
    astrape.deposit(depositAmount, slotCountMap[depositPeriod]);
  };

  const receiveAmount = useMemo(() => {
    return (
      (astrape.poolConfig?.priceFactor || 100_000) *
      depositAmount *
      (astrape.poolConfig?.baseInterestRate || 0.1) *
      (1 - commissionRate / 100)
    );
  }, [depositAmount, commissionRate, astrape.poolConfig?.priceFactor]);

  const maxRisk = useMemo(() => {
    return (
      (astrape.poolConfig?.priceFactor || 100_000) *
      depositAmount *
      Math.max(
        0,
        (astrape.poolConfig?.baseInterestRate || 0.1) *
          2 *
          ((40 - commissionRate) / 100)
      )
    );
  }, [
    commissionRate,
    astrape.poolConfig?.priceFactor,
    depositAmount,
    astrape.poolConfig?.baseInterestRate,
  ]);

  return (
    <main className="page-content">
      <motion.div
        className="page__title"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
      >
        <Icon name="Claim" />
        <span>Claim</span>
      </motion.div>
      <div className="page-widget lg:!-mt-6">
        <section>
          <h1 className="text-2xl font-bold">Amount you will receive NOW</h1>
          <h1 className="text-4xl font-bold">
            {receiveAmount.toLocaleString()} USDC
          </h1>
          <div>APY 10%</div>
        </section>

        <div className="flex w-[600px] flex-col gap-4 rounded-t-md bg-white p-4">
          {step === "amount-and-period" && (
            <AmountAndPeriodStep
              depositAmount={depositAmount}
              depositPeriod={depositPeriod}
              setDepositPeriod={setDepositPeriod}
              onChangeDepositAmount={onChangeDepositAmount}
              onClickNext={() => setStep("commission")}
            />
          )}
          {step === "commission" && (
            <CommissionStep
              commissionRate={commissionRate}
              onChangeCommissionRate={onChangeCommissionRate}
              onClickBack={() => setStep("amount-and-period")}
              onClickDeposit={onClickDeposit}
              maxRisk={maxRisk}
            />
          )}
        </div>
      </div>
    </main>
  );
}

function AmountAndPeriodStep({
  depositAmount,
  depositPeriod,
  setDepositPeriod,
  onChangeDepositAmount,
  onClickNext,
}: {
  depositAmount: number;
  depositPeriod: DepositPeriod;
  setDepositPeriod: (period: DepositPeriod) => void;
  onChangeDepositAmount: (event: React.ChangeEvent<HTMLInputElement>) => void;
  onClickNext: () => void;
}) {
  return (
    <>
      <div className="flex w-full flex-col">
        <div className="flex w-full flex-row">
          <div className="flex w-full flex-col">
            <div className="flex w-full flex-row">
              <input
                type="number"
                value={depositAmount}
                onChange={onChangeDepositAmount}
                className="w-[36px] appearance-none border-none bg-transparent outline-none [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
              />
              <h2> zBTC</h2>
            </div>
            <span>
              Deposit for {slotCountMap[depositPeriod].toLocaleString()} slots
            </span>
          </div>
          <div className="ml-auto flex flex-col">
            <select
              value={depositPeriod}
              onChange={(e) =>
                setDepositPeriod(e.target.value as DepositPeriod)
              }
              className="w-[120px] rounded-md border border-gray-600 bg-transparent p-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              {Object.entries(depositPeriodsDisplayText).map(
                ([value, text]) => (
                  <option
                    key={value}
                    value={value}
                    className="bg-gray-800 text-white"
                  >
                    {text}
                  </option>
                )
              )}
            </select>
          </div>
        </div>
        <input
          type="range"
          min={0.01}
          max={1}
          color="gray"
          step={0.01}
          value={depositAmount}
          onChange={onChangeDepositAmount}
        />
      </div>
      <button
        className="w-full rounded-md bg-blue-500 p-2 text-white hover:bg-blue-600"
        onClick={onClickNext}
      >
        Next
      </button>
    </>
  );
}

function CommissionStep({
  commissionRate,
  onChangeCommissionRate,
  onClickBack,
  onClickDeposit,
  maxRisk,
}: {
  commissionRate: number;
  onChangeCommissionRate: (event: React.ChangeEvent<HTMLInputElement>) => void;
  onClickBack: () => void;
  onClickDeposit: () => void;
  maxRisk: number;
}) {
  return (
    <>
      <button onClick={onClickBack}>Back</button>
      <h1 className="text-2xl font-bold">Adjust Your Risk</h1>
      <span>
        You can adjust your risk by changing the commission rate. The higher the
        commission rate, the lower the risk.
      </span>
      <h3>Maximum risk: {maxRisk.toLocaleString()} USDC</h3>
      <div>
        <input
          type="range"
          min={20}
          max={40}
          step={1}
          value={commissionRate}
          onChange={onChangeCommissionRate}
        />
      </div>
      <button
        className="w-full rounded-md bg-blue-500 p-2 text-white hover:bg-blue-600"
        onClick={onClickDeposit}
      >
        Receive USDC NOW
      </button>
    </>
  );
}
