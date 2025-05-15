"use client";

import { useRouter } from "next/navigation";
import { useState, useEffect, useCallback } from "react";
import { useAstrape } from "@/hooks/astrape/useAstrape";
import Button from "@/components/Button/Button";
import Icon from "@/components/Icons";

export default function Home() {
  const router = useRouter();
  const astrape = useAstrape();
  const { isConfigLoading, config } = astrape;
  const [amount, setAmount] = useState(100000);
  const [period, setPeriod] = useState(3);
  const [interestAmount, setInterestAmount] = useState(0);

  const handleGetStarted = () => {
    router.push("/mint");
  };

  const calculateAdvanceInterest = useCallback(
    (amountUsd: number, periodMonths: number): number => {
      const optimalApy = (config?.baseInterestRate ?? 0) / 100;
      const periodReturnRate = optimalApy * (periodMonths / 12);
      const grossReturn = amountUsd * periodReturnRate;

      const netReturn = grossReturn * 0.8;

      const discountFactor = 1 / (1 + periodReturnRate);
      const advanceInterest = netReturn * discountFactor;

      return advanceInterest;
    },
    [config?.baseInterestRate]
  );

  useEffect(() => {
    const interest = calculateAdvanceInterest(amount, period);
    setInterestAmount(Math.round(interest));
  }, [amount, period, config?.baseInterestRate, calculateAdvanceInterest]);

  const formatCurrency = (value: number): string => {
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency: "USD",
      maximumFractionDigits: 0,
    }).format(value);
  };

  if (isConfigLoading || !config) {
    return (
      <div className="flex h-screen w-full items-center justify-center">
        <div className="flex flex-col items-center gap-4">
          <div className="h-12 w-12 animate-spin rounded-full border-4 border-primary-apollo border-t-transparent"></div>
          <p className="text-lg font-medium text-shade-primary">
            Loading configuration...
          </p>
        </div>
      </div>
    );
  }

  return (
    <main className="w-full">
      <section className="relative overflow-hidden px-20 py-20">
        <div className="absolute inset-0 bg-gradient-to-b from-primary-apollo/5 to-transparent" />
        <div className="absolute -right-20 top-20 h-64 w-64 rounded-full bg-primary-apollo/5 blur-3xl md:h-96 md:w-96" />
        <div className="absolute -left-20 bottom-20 h-64 w-64 rounded-full bg-primary-apollo/5 blur-3xl md:h-96 md:w-96" />

        <div className="relative z-10 mx-auto max-w-7xl">
          <div className="grid grid-cols-1 items-center gap-8 md:grid-cols-2">
            <div className="flex flex-col items-start">
              <h1 className="mb-12 text-4xl font-bold text-shade-primary">
                <span className="text-6xl text-primary-apollo">
                  Deposit BTC
                </span>
                <br />
                <span className="mt-4 inline-block text-5xl">
                  & Claim your Yield Now
                </span>
              </h1>
              <p className="mb-8 text-xl text-shade-secondary md:text-xl">
                Receive your yield upfront.
                <span className="ml-2 rounded-lg bg-primary-apollo/10 px-3 py-1 font-bold text-primary-apollo">
                  Cash Now, Sell Never.
                </span>
              </p>
            </div>

            <div className="mx-auto w-full max-w-md rounded-2xl border border-primary-apollo/10 bg-white p-6 shadow-lg md:ml-auto md:mr-0">
              <h2 className="mb-6 text-center text-2xl font-bold text-shade-primary">
                Interest Calculator
              </h2>
              <div className="mb-6">
                <div className="mb-2 flex items-center justify-between">
                  <label className="font-medium text-shade-primary">
                    Deposit Amount
                  </label>
                  <span className="text-xl font-bold text-primary-apollo">
                    {formatCurrency(amount)}
                  </span>
                </div>
                <input
                  type="range"
                  min="10000"
                  max="1000000"
                  step="10000"
                  value={amount}
                  onChange={(e) => setAmount(Number(e.target.value))}
                  className="h-2 w-full cursor-pointer appearance-none rounded-lg bg-primary-apollo/20"
                />
                <div className="mt-1 flex justify-between text-xs text-shade-secondary">
                  <span>$10,000</span>
                  <span>$1,000,000</span>
                </div>
              </div>

              <div className="mb-6">
                <label className="mb-2 block font-medium text-shade-primary">
                  Deposit Period
                </label>
                <div className="grid grid-cols-3 gap-2">
                  {[1, 3, 6].map((months) => (
                    <button
                      key={months}
                      onClick={() => setPeriod(months)}
                      className={`rounded-xl border px-3 py-2 text-sm ${
                        period === months
                          ? "border-primary-apollo bg-primary-apollo text-white"
                          : "border-primary-apollo/20 bg-white text-shade-primary hover:border-primary-apollo/50"
                      } font-medium transition-all`}
                    >
                      {months} {months === 1 ? "Month" : "Months"}
                    </button>
                  ))}
                </div>
              </div>

              <div className="rounded-xl bg-gradient-to-r from-primary-apollo/10 to-primary-apollo/5 p-5">
                <div className="mb-4 text-center">
                  <span className="mb-1 block text-shade-secondary">
                    You&apos;ll Instantly Receive
                  </span>
                  <span className="text-3xl font-bold text-primary-apollo">
                    {formatCurrency(interestAmount)}
                  </span>
                  <span className="mt-1 block text-shade-secondary">
                    in USDC
                  </span>
                </div>
                <div className="flex justify-center">
                  <Button
                    label="Cash Now"
                    type="primary"
                    size="medium"
                    onClick={handleGetStarted}
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section className="px-4 py-16">
        <div className="mx-auto max-w-7xl">
          <h2 className="mb-6 text-center text-3xl font-bold text-shade-primary">
            How It Works
          </h2>
          <p className="mx-auto mb-12 max-w-3xl text-center text-shade-secondary">
            Deposit BTC and earn interest in just a few simple steps.
          </p>
          <div className="mb-16 grid grid-cols-1 gap-6 md:grid-cols-3">
            <div className="rounded-2xl border border-primary-apollo/10 bg-white p-6 shadow-sm">
              <div className="flex items-start gap-4">
                <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-primary-apollo font-bold text-white">
                  1
                </div>
                <div className="flex-1">
                  <div className="mb-2 flex items-center gap-2">
                    <h3 className="text-xl font-semibold text-shade-primary">
                      Deposit zBTC, Get USDC
                    </h3>
                    <Icon
                      name="zbtc"
                      size={18}
                      className="text-primary-apollo"
                    />
                  </div>
                  <p className="text-shade-secondary">
                    No waiting for periodic interest payments. Receive your
                    interest in USDC upfront when you deposit.
                  </p>
                </div>
              </div>
            </div>

            <div className="rounded-2xl border border-primary-apollo/10 bg-white p-6 shadow-sm">
              <div className="flex items-start gap-4">
                <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-primary-apollo font-bold text-white">
                  2
                </div>
                <div className="flex-1">
                  <div className="mb-2 flex items-center gap-2">
                    <h3 className="text-xl font-semibold text-shade-primary">
                      Monitor in the Dashboard
                    </h3>
                    <Icon
                      name="Portfolio"
                      size={18}
                      className="text-primary-apollo"
                    />
                  </div>
                  <p className="text-shade-secondary">
                    Track your assets, earnings, and maturity dates in real-time
                    through our intuitive dashboard.
                  </p>
                </div>
              </div>
            </div>

            <div className="rounded-2xl border border-primary-apollo/10 bg-white p-6 shadow-sm">
              <div className="flex items-start gap-4">
                <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-primary-apollo font-bold text-white">
                  3
                </div>
                <div className="flex-1">
                  <div className="mb-2 flex items-center gap-2">
                    <h3 className="text-xl font-semibold text-shade-primary">
                      Withdraw on Maturity
                    </h3>
                    <Icon
                      name="Withdraw02"
                      size={18}
                      className="text-primary-apollo"
                    />
                  </div>
                  <p className="text-shade-secondary">
                    Get your BTC back at the end of the term. Early withdrawal
                    options available with a fee.
                  </p>
                </div>
              </div>
            </div>
          </div>

          <div className="grid grid-cols-1 items-center gap-12 md:grid-cols-2">
            <div>
              <h3 className="mb-6 text-2xl font-bold text-shade-primary">
                Interest Claims & Dashboard
              </h3>
              <p className="mb-6 text-lg text-shade-secondary">
                Claim your interest immediately and monitor your assets
              </p>
              <ul className="space-y-4">
                <li className="flex items-start">
                  <Icon
                    name="Tick"
                    size={18}
                    className="mr-3 mt-1 text-primary-apollo"
                  />
                  <div>
                    <span className="block font-medium text-shade-primary">
                      Check claimable interest anytime
                    </span>
                    <span className="text-sm text-shade-secondary">
                      View your earnings in real-time
                    </span>
                  </div>
                </li>
                <li className="flex items-start">
                  <Icon
                    name="Tick"
                    size={18}
                    className="mr-3 mt-1 text-primary-apollo"
                  />
                  <div>
                    <span className="block font-medium text-shade-primary">
                      Monitor asset value in real-time
                    </span>
                    <span className="text-sm text-shade-secondary">
                      Track your BTC and zBTC values
                    </span>
                  </div>
                </li>
                <li className="flex items-start">
                  <Icon
                    name="Tick"
                    size={18}
                    className="mr-3 mt-1 text-primary-apollo"
                  />
                  <div>
                    <span className="block font-medium text-shade-primary">
                      Manage maturity dates
                    </span>
                    <span className="text-sm text-shade-secondary">
                      Set reminders for deposit maturities
                    </span>
                  </div>
                </li>
              </ul>
            </div>
            <div className="rounded-2xl border border-primary-apollo/10 bg-white p-6 shadow-lg">
              <div className="mb-4 rounded-xl bg-gradient-to-r from-primary-apollo/10 to-primary-apollo/5 p-4">
                <h3 className="mb-2 text-xl font-semibold text-shade-primary">
                  Example:
                </h3>
                <div className="mb-2 flex items-center justify-between">
                  <span className="text-shade-secondary">Deposit Amount</span>
                  <span className="font-medium">1 BTC</span>
                </div>
                <div className="mb-2 flex items-center justify-between">
                  <span className="text-shade-secondary">Deposit Period</span>
                  <span className="font-medium">6 months</span>
                </div>
                <div className="mb-2 flex items-center justify-between">
                  <span className="text-shade-secondary">Upfront Interest</span>
                  <span className="font-medium text-green-600">
                    +$77,195 USDC
                  </span>
                </div>
              </div>
              <div className="rounded-xl bg-primary-apollo/5 p-4">
                <div className="text-center">
                  <span className="mb-1 block text-shade-secondary">
                    Annual Interest Rate
                  </span>
                  <span className="text-3xl font-bold text-shade-primary">
                    up to 21.32%
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section className="mx-4 my-12 rounded-3xl bg-gradient-to-r from-primary-apollo/10 to-primary-apollo/20 px-4 py-20 md:mx-8">
        <div className="mx-auto max-w-5xl text-center">
          <h2 className="mb-6 text-3xl font-bold text-shade-primary md:text-4xl">
            Deposit BTC and Earn Interest Now
          </h2>
          <p className="mx-auto mb-10 max-w-3xl text-lg text-shade-secondary">
            Don&apos;t wait any longer. Deposit now and claim your interest
            immediately.
          </p>
          <div className="flex justify-center">
            <Button
              label="Get Started"
              type="primary"
              size="large"
              onClick={handleGetStarted}
            />
          </div>
        </div>
      </section>
    </main>
  );
}
