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
import DepositSuccessModal from "./modal";
import { useWallet } from "@solana/wallet-adapter-react";
import { useConnection } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import BigNumber from "bignumber.js";
import useSWR from "swr";
import { getAccount, getAssociatedTokenAddress } from "@solana/spl-token";
import { BTC_DECIMALS } from "@/utils/constant";
import { formatValue } from "@/utils/format";

// Custom hook to fetch zBTC balance with specific token mint
const useZbtcBalance = (walletPublicKey: PublicKey | null) => {
  const { connection } = useConnection();
  const ZBTC_MINT = new PublicKey(
    "91AgzqSfXnCq6AJm5CPPHL3paB25difEJ1TfSnrFKrf"
  );

  const balanceFetcher = async (publicKey: PublicKey, mint: PublicKey) => {
    try {
      const ata = await getAssociatedTokenAddress(mint, publicKey, true);
      const accountData = await getAccount(connection, ata);
      return new BigNumber(accountData.amount.toString());
    } catch {
      return new BigNumber(0);
    }
  };

  const { data, isLoading } = useSWR<BigNumber>(
    walletPublicKey ? [walletPublicKey.toBase58(), "zbtc-balance"] : null,
    async ([pubkeyStr]) => {
      return balanceFetcher(new PublicKey(pubkeyStr), ZBTC_MINT);
    },
    {
      refreshInterval: 30000,
      dedupingInterval: 30000,
    }
  );

  return {
    balance: data ?? new BigNumber(0),
    isLoading,
  };
};

type DepositPeriod = "1M" | "3M" | "6M";

const depositPeriodsDisplayText: Record<DepositPeriod, string> = {
  "1M": "1 Month",
  "3M": "3 Months",
  "6M": "6 Months",
};

export const slotCountMap: Record<DepositPeriod, number> = {
  "1M": 5890909,
  "3M": 17672727,
  "6M": 35345454,
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

// Minimum risk-buffer (0-1) so that the conservative APY (after commission)
// equals the 3 % worst-case APY.  This guarantees the displayed "Target APY"
// falls back to exactly 3 % when the slider is at its maximum.
function calculateMinRiskBuffer(
  currentAPY: number,
  {
    worstCaseAPY = 0.03, // 3% (gross)
    commissionRate = 0.2, // 20%
  }: {
    worstCaseAPY?: number;
    commissionRate?: number;
  } = {}
) {
  const effectiveCurrent = currentAPY * (1 - commissionRate);
  if (effectiveCurrent <= 0) return 1;
  const buffer = 1 - worstCaseAPY / effectiveCurrent;
  return Math.max(0, Math.min(1, buffer));
}

export default function DepositPage() {
  const [step, setStep] = useState<"amount-and-period" | "risk-buffer">(
    "amount-and-period"
  );
  const [depositAmount, setDepositAmount] = useState(1);
  const [depositPeriod, setDepositPeriod] = useState<DepositPeriod>("3M");
  const [riskBuffer, setRiskBuffer] = useState(0);
  const [showSuccessModal, setShowSuccessModal] = useState(false);
  const [transactionData, setTransactionData] = useState<{
    depositAmount: number;
    depositPeriod: string;
    receiveAmount: number;
    transactionSignature?: string;
  }>();

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

  const onClickDeposit = async () => {
    // Default commission rate is 20% (20)
    const commissionRate = 20;
    try {
      const signature = await astrape.deposit(
        depositAmount,
        slotCountMap[depositPeriod],
        commissionRate * (1 + riskBuffer / 100)
      );

      setTransactionData({
        depositAmount,
        depositPeriod,
        receiveAmount,
        transactionSignature: signature,
      });
      setShowSuccessModal(true);
    } catch (error) {
      console.error("Deposit failed:", error);
      // You might want to add error handling here
    }
  };

  const usdAmount = useMemo(() => {
    return depositAmount * (astrape.config?.priceFactor || 100000);
  }, [depositAmount, astrape.config?.priceFactor]);

  const currentAPY = useMemo(() => {
    return calculateOptimalAPY(usdAmount);
  }, [usdAmount]);

  const minRiskBuffer = useMemo(() => {
    return calculateMinRiskBuffer(currentAPY);
  }, [currentAPY]);

  const conservativeAPY = useMemo(() => {
    const commissionRate = 0.2;
    return currentAPY * (1 - commissionRate) * (1 - riskBuffer / 100);
  }, [currentAPY, riskBuffer]);

  const receiveAmount = useMemo(() => {
    const periodRate =
      conservativeAPY * (Number(depositPeriod.replace("M", "")) / 12);
    const discountFactor = 1 / (1 + periodRate);
    return usdAmount * periodRate * discountFactor;
  }, [usdAmount, conservativeAPY, depositPeriod]);

  const scenarioAnalysis = useMemo(() => {
    const periodMonths = Number(depositPeriod.replace("M", ""));
    const commissionRate = 0.2;

    const scenarios = [
      { name: "Worst (3%)", apy: 0.03, color: "#ef4444" },
      { name: "Normal (10%)", apy: 0.1, color: "#f97316" },
      { name: "Good (15%)", apy: 0.15, color: "#22c55e" },
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
    <main className="w-full overflow-hidden">
      <div className="relative px-4 py-12 md:px-20 md:py-16">
        <div className="absolute inset-0 bg-gradient-to-b from-primary-apollo/5 to-transparent" />
        <div className="absolute -right-20 top-20 h-64 w-64 rounded-full bg-primary-apollo/5 blur-3xl md:h-96 md:w-96" />
        <div className="absolute -left-20 bottom-20 h-64 w-64 rounded-full bg-primary-apollo/5 blur-3xl md:h-96 md:w-96" />

        <div className="relative z-10 mx-auto max-w-7xl">
          <motion.div
            className="mb-8 flex items-center gap-3 text-3xl font-bold text-shade-primary"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
          >
            <Icon name="Claim" className="text-primary-apollo" />
            <span>Advance Interest Claim</span>
          </motion.div>

          <div
            className={`grid grid-cols-1 gap-8 ${step === "amount-and-period" ? "md:grid-cols-2" : ""}`}
          >
            <div className="rounded-2xl border border-primary-apollo/10 bg-white p-6 shadow-lg">
              <h1 className="mb-2 text-2xl font-bold text-shade-primary">
                Amount you will receive NOW
              </h1>
              <h1 className="mb-4 text-4xl font-bold text-primary-apollo">
                $
                {receiveAmount.toLocaleString(undefined, {
                  maximumFractionDigits: 0,
                })}{" "}
                USDC
              </h1>
              <div className="rounded-xl bg-gradient-to-r from-primary-apollo/10 to-primary-apollo/5 p-4 text-center">
                <span className="block text-shade-secondary">Target APY</span>
                <span className="text-2xl font-bold text-shade-primary">
                  {(conservativeAPY * 100).toFixed(2)}%
                </span>
              </div>
            </div>

            <div className="flex flex-col gap-4 rounded-2xl border border-primary-apollo/10 bg-white p-6 shadow-lg">
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
        </div>
      </div>

      <DepositSuccessModal
        isOpen={showSuccessModal}
        onClose={() => setShowSuccessModal(false)}
        transactionData={transactionData}
      />
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
  const { publicKey } = useWallet();
  const { balance: zbtcBalance, isLoading: isZbtcBalanceLoading } =
    useZbtcBalance(publicKey);
  const formattedZbtcBalance = useMemo(() => {
    return formatValue(zbtcBalance.div(10 ** BTC_DECIMALS), 6);
  }, [zbtcBalance]);

  const isDepositValid = useMemo(() => {
    if (!publicKey) return false;
    if (isZbtcBalanceLoading) return false;
    return depositAmount <= zbtcBalance.div(10 ** BTC_DECIMALS).toNumber();
  }, [publicKey, depositAmount, zbtcBalance, isZbtcBalanceLoading]);

  const buttonText = useMemo(() => {
    if (!publicKey) return "Connect Wallet to Continue";
    if (depositAmount > zbtcBalance.div(10 ** BTC_DECIMALS).toNumber()) {
      return "Insufficient Balance";
    }
    return "Next";
  }, [publicKey, depositAmount, zbtcBalance]);

  return (
    <>
      <h2 className="mb-4 text-2xl font-bold text-shade-primary">
        Deposit Details
      </h2>
      <div className="mb-6 flex w-full flex-col">
        <div className="mb-4 flex w-full flex-row justify-between">
          <div className="flex flex-col">
            <div className="flex items-end gap-2">
              <input
                type="number"
                value={depositAmount}
                onChange={onChangeDepositAmount}
                className="w-[80px] appearance-none border-none bg-transparent text-3xl font-bold text-primary-apollo outline-none [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
              />
              <h2 className="text-xl font-bold text-shade-primary">zBTC</h2>
            </div>
            <span className="text-shade-secondary">
              ≈ ${usdAmount.toLocaleString()} USD
            </span>
          </div>
          <div className="flex flex-col items-end justify-end">
            <div className="flex flex-col items-end gap-1">
              <span className="text-sm text-shade-secondary">
                Deposit Period
              </span>
              <select
                value={depositPeriod}
                onChange={(e) =>
                  setDepositPeriod(e.target.value as DepositPeriod)
                }
                className="w-[120px] rounded-md border border-primary-apollo/20 bg-white p-2 text-shade-primary focus:border-primary-apollo focus:outline-none"
              >
                {Object.entries(depositPeriodsDisplayText).map(
                  ([value, text]) => (
                    <option
                      key={value}
                      value={value}
                      className="bg-white text-shade-primary"
                    >
                      {text}
                    </option>
                  )
                )}
              </select>
            </div>
          </div>
        </div>

        {/* Wallet balance indicator */}
        <div className="mb-3 mt-1 flex items-center justify-end">
          <div className="flex items-center gap-1 text-sm text-shade-secondary">
            <Icon
              name="WalletSmall"
              size={14}
              className="text-primary-apollo"
            />
            <span>
              {isZbtcBalanceLoading ? (
                "Loading balance..."
              ) : publicKey ? (
                <>
                  Available:{" "}
                  <span className="font-medium text-primary-apollo">
                    {formattedZbtcBalance} zBTC
                  </span>
                </>
              ) : (
                "Connect wallet to see balance"
              )}
            </span>
          </div>
        </div>

        <div className="mb-2 mt-4">
          <span className="text-sm font-medium text-shade-secondary">
            Adjust Amount
          </span>
        </div>
        <input
          type="range"
          min={0.001}
          max={10}
          step={0.001}
          value={depositAmount}
          onChange={onChangeDepositAmount}
          className="h-2 w-full cursor-pointer appearance-none rounded-lg bg-primary-apollo/20"
        />
        <div className="mt-1 flex justify-between text-xs text-shade-secondary">
          <span>0.001 zBTC</span>
          <span>10 zBTC</span>
        </div>
      </div>

      <div className="mt-4 rounded-xl bg-gradient-to-r from-primary-apollo/10 to-primary-apollo/5 p-4">
        <span className="mb-1 block text-center text-shade-secondary">
          You will deposit for
        </span>
        <div className="text-center text-xl font-bold text-shade-primary">
          {Number(depositPeriod.replace("M", ""))}{" "}
          {Number(depositPeriod.replace("M", "")) === 1 ? "month" : "months"}
        </div>
      </div>

      <button
        className={`mt-6 w-full rounded-xl py-3 text-white transition-all ${
          isDepositValid
            ? "bg-primary-apollo hover:bg-primary-apollo/90"
            : "cursor-not-allowed bg-primary-apollo/50"
        }`}
        onClick={onClickNext}
        disabled={!isDepositValid}
      >
        {buttonText}
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
        className="mb-4 flex items-center text-primary-apollo hover:text-primary-apollo/80"
      >
        <Icon name="ChevronDownSmall" size={14} className="mr-1 rotate-90" />{" "}
        Back
      </button>

      <h1 className="mb-2 text-2xl font-bold text-shade-primary">
        Adjust Your Risk Buffer
      </h1>
      <p className="mb-4 text-shade-secondary">
        Commission rate is fixed at 20%. Adjust risk buffer to balance between
        advance interest and principal protection.
      </p>

      <div className="mb-6 flex flex-col gap-3 rounded-xl bg-gradient-to-r from-primary-apollo/10 to-primary-apollo/5 p-4">
        <div className="flex justify-between">
          <span className="text-shade-secondary">Risk Buffer</span>
          <span className="font-medium text-shade-primary">
            {riskBuffer.toFixed(1)}%
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-shade-secondary">Target APY</span>
          <span className="font-medium text-shade-primary">
            {(conservativeAPY * 100).toFixed(2)}%
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-shade-secondary">Principal Protection</span>
          <span className="font-medium text-shade-primary">
            {(worstCase?.protection || 0).toFixed(1)}%
          </span>
        </div>
        {riskBuffer >= maxRiskBuffer - 0.1 && (
          <div className="mt-2 rounded-lg bg-green-50 p-2 text-center font-semibold text-green-600">
            ✓ Principal 99.9% protected even in worst case (3% APY)
          </div>
        )}
      </div>

      {/* Scenario Analysis Bar Chart */}
      <div className="mb-6">
        <h3 className="mb-3 text-lg font-semibold text-shade-primary">
          Scenario Analysis
        </h3>
        <div className="h-64 rounded-xl border border-primary-apollo/10 bg-white p-2 shadow-sm">
          <ResponsiveContainer width="100%" height="100%">
            <ComposedChart
              data={scenarioAnalysis}
              margin={{ top: 20, right: 30, left: 20, bottom: 5 }}
            >
              <CartesianGrid strokeDasharray="3 3" opacity={0.3} />
              <XAxis dataKey="name" />
              <YAxis />
              <Tooltip
                formatter={(value: number, name: string) => {
                  if (name === "protection") return `${value.toFixed(1)}%`;
                  return `$${value.toLocaleString()}`;
                }}
                contentStyle={{
                  borderRadius: "8px",
                  border: "1px solid rgba(0, 0, 0, 0.1)",
                  boxShadow: "0 4px 6px rgba(0, 0, 0, 0.05)",
                }}
              />
              <Legend />
              <Bar
                dataKey="netReturn"
                name="Net Return"
                fill="#22c55e"
                radius={[4, 4, 0, 0]}
              />
              <Bar
                dataKey="principalLoss"
                name="Principal Loss Risk"
                fill="#ef4444"
                radius={[4, 4, 0, 0]}
              />
            </ComposedChart>
          </ResponsiveContainer>
        </div>
      </div>

      <div className="mb-6">
        <div className="mb-2 flex justify-between text-sm">
          <span className="font-medium text-shade-primary">Risk Buffer</span>
          <span className="font-medium text-primary-apollo">
            {riskBuffer.toFixed(1)}%
          </span>
        </div>
        <input
          type="range"
          min={0}
          max={maxRiskBuffer}
          step={0.1}
          value={riskBuffer}
          onChange={onChangeRiskBuffer}
          className="h-2 w-full cursor-pointer appearance-none rounded-lg bg-primary-apollo/20"
        />
        <div className="mt-1 flex justify-between text-xs text-shade-secondary">
          <span>0% (Max Risk)</span>
          <span>{maxRiskBuffer.toFixed(1)}% (Principal Protected)</span>
        </div>
      </div>

      <button
        className="mt-4 w-full rounded-xl bg-primary-apollo py-3 text-white transition-all hover:bg-primary-apollo/90"
        onClick={onClickDeposit}
      >
        Receive ${receiveAmount.toLocaleString()} USDC NOW
      </button>
    </>
  );
}
