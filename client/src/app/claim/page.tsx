"use client";

import { motion } from "framer-motion";
import Icon from "@/components/Icons";
import { useMemo, useState } from "react";
import { useAstrape } from "@/hooks/astrape/useAstrape";
import {
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ComposedChart,
  ResponsiveContainer,
} from "recharts";

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

// Calculate optimal APY based on amount in USD
function calculateOptimalAPY(usdAmount: number) {
  if (usdAmount <= 10000000) {
    // 10M USD
    return 0.2132; // 21.32%
  } else {
    const wfragAmount = 10000000;
    const fragAmount = usdAmount - 10000000;
    return (wfragAmount * 0.2132 + fragAmount * 0.1432) / usdAmount;
  }
}

// Calculate minimum risk buffer for full principal protection
function calculateMinRiskBuffer(currentAPY: number) {
  const worstCaseAPY = 0.03; // 3%
  const fixedCommission = 0.2; // 20%
  return Math.max(
    0,
    Math.min(1, 1 - worstCaseAPY / (currentAPY * (1 - fixedCommission)))
  );
}

export default function ClaimPage() {
  const [step, setStep] = useState<"amount-and-period" | "risk-buffer">(
    "amount-and-period"
  );
  const [depositAmount, setDepositAmount] = useState(1);
  const [depositPeriod, setDepositPeriod] = useState<DepositPeriod>("3M");
  const [riskBuffer, setRiskBuffer] = useState(0);

  const astrape = useAstrape();

  const onChangeDepositAmount = (
    event: React.ChangeEvent<HTMLInputElement>
  ) => {
    setDepositAmount(Math.round(event.target.valueAsNumber * 1000) / 1000);
  };

  const onChangeRiskBuffer = (event: React.ChangeEvent<HTMLInputElement>) => {
    const newRate = Math.round(event.target.valueAsNumber * 10) / 10;
    setRiskBuffer(newRate);
  };

  const onClickDeposit = () => {
    astrape.deposit(depositAmount, slotCountMap[depositPeriod]);
  };

  // Convert to USD
  const usdAmount = useMemo(() => {
    return depositAmount * (astrape.poolConfig?.priceFactor || 100000);
  }, [depositAmount, astrape.poolConfig?.priceFactor]);

  // Calculate current APY based on USD amount
  const currentAPY = useMemo(() => {
    return calculateOptimalAPY(usdAmount);
  }, [usdAmount]);

  // Calculate minimum risk buffer for full protection
  const minRiskBuffer = useMemo(() => {
    return calculateMinRiskBuffer(currentAPY);
  }, [currentAPY]);

  // Calculate conservative APY
  const conservativeAPY = useMemo(() => {
    const commissionRate = 0.2;
    return currentAPY * (1 - commissionRate) * (1 - riskBuffer / 100);
  }, [currentAPY, riskBuffer]);

  // Calculate receive amount
  const receiveAmount = useMemo(() => {
    const periodMonths = slotCountMap[depositPeriod];
    const periodRate = conservativeAPY * (periodMonths / 12);
    const discountFactor = 1 / (1 + periodRate);
    return usdAmount * periodRate * discountFactor;
  }, [usdAmount, conservativeAPY, depositPeriod]);

  // Calculate scenario analysis for charts
  const scenarioAnalysis = useMemo(() => {
    const periodMonths = slotCountMap[depositPeriod];
    const commissionRate = 0.2;

    const scenarios = [
      { name: "Worst (3%)", apy: 0.03, color: "#ef4444" },
      { name: "Bad (10%)", apy: 0.1, color: "#f97316" },
      { name: "Normal (15%)", apy: 0.15, color: "#22c55e" },
      { name: "Optimal", apy: currentAPY, color: "#3b82f6" },
    ];

    return scenarios.map((scenario) => {
      const grossReturn = usdAmount * scenario.apy * (periodMonths / 12);
      const netReturn = grossReturn * (1 - commissionRate);
      const principalLoss = Math.max(0, receiveAmount - netReturn);
      const finalPrincipal = usdAmount - principalLoss;
      const additionalInterest = Math.max(0, netReturn - receiveAmount);

      return {
        ...scenario,
        grossReturn,
        netReturn,
        principalLoss,
        finalPrincipal,
        additionalInterest,
        protection: (finalPrincipal / usdAmount) * 100,
      };
    });
  }, [usdAmount, depositPeriod, receiveAmount, currentAPY]);

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
            $
            {receiveAmount.toLocaleString(undefined, {
              maximumFractionDigits: 0,
            })}{" "}
            USDC
          </h1>
          <div>Target APY: {(conservativeAPY * 100).toFixed(2)}%</div>
        </section>

        <div className="flex w-[800px] flex-col gap-4 rounded-t-md bg-white p-4">
          {step === "amount-and-period" && (
            <AmountAndPeriodStep
              depositAmount={depositAmount}
              depositPeriod={depositPeriod}
              usdAmount={usdAmount}
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
              scenarioAnalysis={scenarioAnalysis}
              conservativeAPY={conservativeAPY}
              receiveAmount={receiveAmount}
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
  usdAmount,
  setDepositPeriod,
  onChangeDepositAmount,
  onClickNext,
}: {
  depositAmount: number;
  depositPeriod: DepositPeriod;
  usdAmount: number;
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
            <span>≈ ${usdAmount.toLocaleString()} USD</span>
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
          min={0.001}
          max={10}
          color="gray"
          step={0.001}
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
  scenarioAnalysis,
  conservativeAPY,
  receiveAmount,
}: {
  riskBuffer: number;
  minRiskBuffer: number;
  onChangeRiskBuffer: (event: React.ChangeEvent<HTMLInputElement>) => void;
  onClickBack: () => void;
  onClickDeposit: () => void;
  scenarioAnalysis: {
    name: string;
    apy: number;
    color: string;
    grossReturn: number;
    netReturn: number;
    principalLoss: number;
    finalPrincipal: number;
    additionalInterest: number;
    protection: number;
  }[];
  conservativeAPY: number;
  receiveAmount: number;
}) {
  const worstCase = scenarioAnalysis.find((s) => s.name === "Worst (3%)");
  const maxRiskBuffer = minRiskBuffer * 100;

  return (
    <>
      <button
        onClick={onClickBack}
        className="text-blue-500 hover:text-blue-600"
      >
        ← Back
      </button>
      <h1 className="text-2xl font-bold">Adjust Your Risk Buffer</h1>
      <span>
        Commission rate is fixed at 20%. Adjust risk buffer to balance between
        advance interest and principal protection.
      </span>

      <div className="flex flex-col gap-2 rounded bg-gray-100 p-4">
        <div>Risk Buffer: {riskBuffer.toFixed(1)}%</div>
        <div>Target APY: {(conservativeAPY * 100).toFixed(2)}%</div>
        <div>
          Principal Protection: {(worstCase?.protection || 0).toFixed(1)}%
        </div>
        {riskBuffer >= maxRiskBuffer && (
          <div className="font-semibold text-green-600">
            ✓ Principal 100% protected even in worst case (3% APY)
          </div>
        )}
      </div>

      {/* Scenario Analysis Bar Chart */}
      <div className="mt-6">
        <h3 className="mb-3 text-lg font-semibold">Scenario Analysis</h3>
        <div className="h-64">
          <ResponsiveContainer width="100%" height="100%">
            <ComposedChart
              data={scenarioAnalysis}
              margin={{ top: 20, right: 30, left: 20, bottom: 5 }}
            >
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="name" />
              <YAxis />
              <Tooltip
                formatter={(value: number, name: string) => {
                  if (name === "protection") return `${value.toFixed(1)}%`;
                  return `$${value.toLocaleString()}`;
                }}
              />
              <Legend />
              <Bar dataKey="netReturn" name="Net Return" fill="#22c55e" />
              <Bar
                dataKey="principalLoss"
                name="Principal Loss"
                fill="#ef4444"
              />
            </ComposedChart>
          </ResponsiveContainer>
        </div>
      </div>

      <div className="mt-6 text-center">
        <input
          type="range"
          min={0}
          max={maxRiskBuffer}
          step={0.1}
          value={riskBuffer}
          onChange={onChangeRiskBuffer}
          className="w-full"
        />
        <div className="mt-1 flex justify-between text-xs text-gray-500">
          <span>0% (Max Risk)</span>
          <span>{maxRiskBuffer.toFixed(1)}% (Principal Protected)</span>
        </div>
      </div>

      <button
        className="mt-4 w-full rounded-md bg-blue-500 p-2 text-white hover:bg-blue-600"
        onClick={onClickDeposit}
      >
        Receive ${receiveAmount.toLocaleString()} USDC NOW
      </button>
    </>
  );
}
