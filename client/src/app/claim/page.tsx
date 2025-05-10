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
  "1M": 1,
  "3M": 3,
  "6M": 6,
};

// Calculate optimal APY based on amount
function calculateOptimalAPY(amount: number) {
  if (amount <= 10) {
    // 10M USD (assuming amount is in millions)
    return 0.2132; // 21.32%
  } else {
    const wfragAmount = 10;
    const fragAmount = amount - 10;
    return (wfragAmount * 0.2132 + fragAmount * 0.1432) / amount;
  }
}

// Calculate minimum risk buffer for full principal protection
function calculateMinRiskBuffer(currentAPY: number) {
  const worstCaseAPY = 0.03; // 3%
  const fixedCommission = 0.2; // 20%
  return 1 - worstCaseAPY / (currentAPY * (1 - fixedCommission));
}

export default function ClaimPage() {
  const [step, setStep] = useState<"amount-and-period" | "risk-buffer">(
    "amount-and-period"
  );
  const [depositAmount, setDepositAmount] = useState(1);
  const [depositPeriod, setDepositPeriod] = useState<DepositPeriod>("3M");
  const [riskBuffer, setRiskBuffer] = useState(70); // Default to 70%

  const astrape = useAstrape();

  const onChangeDepositAmount = (
    event: React.ChangeEvent<HTMLInputElement>
  ) => {
    setDepositAmount(Math.round(event.target.valueAsNumber * 100) / 100);
  };

  const onChangeRiskBuffer = (event: React.ChangeEvent<HTMLInputElement>) => {
    const newRate = Math.round(event.target.valueAsNumber * 100) / 100;
    setRiskBuffer(Math.max(30, Math.min(90, newRate)));
  };

  const onClickDeposit = () => {
    astrape.deposit(depositAmount, slotCountMap[depositPeriod]);
  };

  // Calculate current APY based on deposit amount
  const currentAPY = useMemo(() => {
    return calculateOptimalAPY(depositAmount);
  }, [depositAmount]);

  // Calculate minimum risk buffer for full protection
  const minRiskBuffer = useMemo(() => {
    return calculateMinRiskBuffer(currentAPY);
  }, [currentAPY]);

  // Calculate conservative APY
  const conservativeAPY = useMemo(() => {
    const commissionRate = 0.2; // Fixed 20%
    return currentAPY * (1 - commissionRate) * (1 - riskBuffer / 100);
  }, [currentAPY, riskBuffer]);

  // Calculate receive amount (advance interest)
  const receiveAmount = useMemo(() => {
    const periodMonths = slotCountMap[depositPeriod];
    const periodRate = conservativeAPY * (periodMonths / 12);
    const discountFactor = 1 / (1 + periodRate);

    // Convert to USD (assuming depositAmount is in zBTC)
    const usdAmount =
      depositAmount * (astrape.poolConfig?.priceFactor || 50000);

    return usdAmount * periodRate * discountFactor;
  }, [
    depositAmount,
    conservativeAPY,
    depositPeriod,
    astrape.poolConfig?.priceFactor,
  ]);

  // Calculate maximum risk (principal loss in worst case)
  const maxRisk = useMemo(() => {
    const periodMonths = slotCountMap[depositPeriod];
    const worstCaseAPY = 0.03; // 3%
    const commissionRate = 0.2; // Fixed 20%

    // Convert to USD
    const usdAmount =
      depositAmount * (astrape.poolConfig?.priceFactor || 50000);

    // Calculate worst case return
    const worstCaseReturn = usdAmount * worstCaseAPY * (periodMonths / 12);
    const netWorstCaseReturn = worstCaseReturn * (1 - commissionRate);

    // Calculate principal loss if advance interest exceeds worst case return
    return Math.max(0, receiveAmount - netWorstCaseReturn);
  }, [
    depositAmount,
    depositPeriod,
    receiveAmount,
    astrape.poolConfig?.priceFactor,
  ]);

  // Calculate principal protection percentage
  const principalProtection = useMemo(() => {
    const usdAmount =
      depositAmount * (astrape.poolConfig?.priceFactor || 50000);
    return ((usdAmount - maxRisk) / usdAmount) * 100;
  }, [depositAmount, maxRisk, astrape.poolConfig?.priceFactor]);

  return (
    <main className="page-content">
      <motion.div
        className="page__title"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
      >
        <Icon name="Claim" />
        <span>Advance Interest Claim</span>
      </motion.div>
      <div className="page-widget lg:!-mt-6">
        <section>
          <h1 className="text-2xl font-bold">Amount you will receive NOW</h1>
          <h1 className="text-4xl font-bold">
            {receiveAmount.toLocaleString()} USDC
          </h1>
          <div>Conservative APY: {(conservativeAPY * 100).toFixed(2)}%</div>
          <div>Current Optimal APY: {(currentAPY * 100).toFixed(2)}%</div>
        </section>

        <div className="flex w-[600px] flex-col gap-4 rounded-t-md bg-white p-4">
          {step === "amount-and-period" && (
            <AmountAndPeriodStep
              depositAmount={depositAmount}
              depositPeriod={depositPeriod}
              setDepositPeriod={setDepositPeriod}
              onChangeDepositAmount={onChangeDepositAmount}
              onClickNext={() => setStep("risk-buffer")}
            />
          )}
          {step === "risk-buffer" && (
            <RiskBufferStep
              riskBuffer={riskBuffer}
              minRiskBuffer={minRiskBuffer}
              onChangeRiskBuffer={onChangeRiskBuffer}
              onClickBack={() => setStep("amount-and-period")}
              onClickDeposit={onClickDeposit}
              maxRisk={maxRisk}
              principalProtection={principalProtection}
              conservativeAPY={conservativeAPY}
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
                className="w-[80px] appearance-none border-none bg-transparent outline-none [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
              />
              <h2> zBTC</h2>
            </div>
            <span>Deposit for {slotCountMap[depositPeriod]} months</span>
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
          max={100}
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

function RiskBufferStep({
  riskBuffer,
  minRiskBuffer,
  onChangeRiskBuffer,
  onClickBack,
  onClickDeposit,
  maxRisk,
  principalProtection,
  conservativeAPY,
}: {
  riskBuffer: number;
  minRiskBuffer: number;
  onChangeRiskBuffer: (event: React.ChangeEvent<HTMLInputElement>) => void;
  onClickBack: () => void;
  onClickDeposit: () => void;
  maxRisk: number;
  principalProtection: number;
  conservativeAPY: number;
}) {
  return (
    <>
      <button onClick={onClickBack}>Back</button>
      <h1 className="text-2xl font-bold">Adjust Your Risk Buffer</h1>
      <span>
        Commission rate is fixed at 20%. You can adjust your risk by changing
        the risk buffer. Higher risk buffer = Lower risk, Lower advance
        interest.
      </span>
      <div className="flex flex-col gap-2">
        <div>Risk Buffer: {riskBuffer}%</div>
        <div>Conservative APY: {(conservativeAPY * 100).toFixed(2)}%</div>
        <div>Principal Protection: {principalProtection.toFixed(1)}%</div>
        <div className="text-red-500">
          Maximum Principal Loss: {maxRisk.toLocaleString()} USDC
        </div>
        {riskBuffer < minRiskBuffer && (
          <div className="text-yellow-500">
            Warning: Risk buffer below recommended minimum (
            {minRiskBuffer.toFixed(1)}%) for full principal protection
          </div>
        )}
      </div>
      <div>
        <input
          type="range"
          min={30}
          max={90}
          step={1}
          value={riskBuffer}
          onChange={onChangeRiskBuffer}
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
