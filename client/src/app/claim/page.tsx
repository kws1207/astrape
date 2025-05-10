"use client";

import { motion } from "framer-motion";

import Icon from "@/components/Icons";
import { useState } from "react";

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
  const [depositAmount, setDepositAmount] = useState(1);
  const [depositPeriod, setDepositPeriod] = useState<DepositPeriod>("3M");

  const onChangeDepositAmount = (
    event: React.ChangeEvent<HTMLInputElement>
  ) => {
    setDepositAmount(Math.round(event.target.valueAsNumber * 100) / 100);
  };

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
          <h1 className="text-4xl font-bold">300 USDC</h1>
          <div>APY 10%</div>
        </section>
        <section>
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
                  Deposit for {slotCountMap[depositPeriod].toLocaleString()}{" "}
                  slots
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
        </section>
      </div>
    </main>
  );
}
