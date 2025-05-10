"use client";

import { useRouter } from "next/navigation";
import { useState, useEffect } from "react";
import Button from "@/components/Button/Button";
import Icon from "@/components/Icons";
import { IconName } from "@/components/Icons/icons";

export default function Home() {
  const router = useRouter();
  const [amount, setAmount] = useState(100000); // 초기 금액: $100,000
  const [period, setPeriod] = useState(3); // 초기 기간: 3개월
  const [interestAmount, setInterestAmount] = useState(0);

  const handleGetStarted = () => {
    router.push("/mint");
  };

  // 선이자 계산 함수
  const calculateOptimalApy = (amountUsd: number): number => {
    if (amountUsd <= 1000000) {
      return 0.2132; // 21.32% for amounts up to 1M
    } else if (amountUsd <= 10000000) {
      const wAmount = Math.min(amountUsd, 10000000);
      const fAmount = Math.max(amountUsd - 10000000, 0);
      return (wAmount * 0.2132 + fAmount * 0.1432) / amountUsd;
    } else {
      return (10000000 * 0.2132 + (amountUsd - 10000000) * 0.1432) / amountUsd;
    }
  };

  const calculateAdvanceInterest = (
    amountUsd: number,
    periodMonths: number
  ): number => {
    // Step 1: 최적 APY 결정
    const optimalApy = calculateOptimalApy(amountUsd);

    // Step 2: 기간별 수익 계산
    const periodReturnRate = optimalApy * (periodMonths / 12);
    const grossReturn = amountUsd * periodReturnRate;

    // Step 3: 수수료 차감 (20%)
    const netReturn = grossReturn * 0.8;

    // Step 4: 선이자 할인 적용
    const discountFactor = 1 / (1 + periodReturnRate);
    const advanceInterest = netReturn * discountFactor;

    return advanceInterest;
  };

  // 입력값 변경 시 이자 다시 계산
  useEffect(() => {
    const interest = calculateAdvanceInterest(amount, period);
    setInterestAmount(Math.round(interest));
  }, [amount, period]);

  // 금액 포맷팅 함수
  const formatCurrency = (value: number): string => {
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency: "USD",
      maximumFractionDigits: 0,
    }).format(value);
  };

  return (
    <main className="w-full">
      {/* 히어로 섹션 */}
      <section className="relative overflow-hidden px-20 py-20">
        {/* 배경 효과 */}
        <div className="absolute inset-0 bg-gradient-to-b from-primary-apollo/5 to-transparent" />
        <div className="absolute -right-20 top-20 h-64 w-64 rounded-full bg-primary-apollo/5 blur-3xl md:h-96 md:w-96" />
        <div className="absolute -left-20 bottom-20 h-64 w-64 rounded-full bg-primary-apollo/5 blur-3xl md:h-96 md:w-96" />

        {/* 컨텐츠 */}
        <div className="relative z-10 mx-auto max-w-7xl">
          <div className="grid grid-cols-1 items-center gap-8 md:grid-cols-2">
            {/* 왼쪽: 타이틀과 설명 */}
            <div className="flex flex-col items-start">
              <h1 className="mb-12 text-4xl font-bold text-shade-primary md:text-5xl lg:text-6xl">
                <span className="text-primary-apollo">Deposit BTC</span>
                <br />
                <span className="mt-4 inline-block">& Earn Interest Now</span>
              </h1>
              <p className="mb-8 text-lg text-shade-secondary md:text-lg">
                <span>
                  Receive immediate interest in USDC for your deposited BTC.
                </span>
                <br />
                <span>Use now, Pay later.</span>
              </p>
            </div>

            {/* 오른쪽: 이자 계산기 */}
            <div className="mx-auto w-full max-w-md rounded-2xl border border-primary-apollo/10 bg-white p-6 shadow-lg md:ml-auto md:mr-0">
              <h2 className="mb-6 text-center text-2xl font-bold text-shade-primary">
                Interest Calculator
              </h2>
              {/* 금액 선택 */}
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

              {/* 기간 선택 */}
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

              {/* 이자 결과 */}
              <div className="rounded-xl bg-gradient-to-r from-primary-apollo/10 to-primary-apollo/5 p-5">
                <div className="mb-4 text-center">
                  <span className="mb-1 block text-shade-secondary">
                    You'll Receive Instantly
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
                    label="Start Earning Now"
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

      {/* 사용 방법 */}
      <section className="px-4 py-16">
        <div className="mx-auto max-w-7xl">
          <h2 className="mb-6 text-center text-3xl font-bold text-shade-primary">
            How It Works
          </h2>
          <p className="mx-auto mb-12 max-w-3xl text-center text-shade-secondary">
            Deposit BTC and earn interest in just a few simple steps.
          </p>
          <div className="mb-16 grid grid-cols-1 gap-6 md:grid-cols-2">
            <div className="rounded-2xl border border-primary-apollo/10 bg-white p-6 shadow-sm">
              <div className="flex items-start gap-4">
                <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-primary-apollo font-bold text-white">
                  1
                </div>
                <div className="flex-1">
                  <div className="mb-2 flex items-center gap-2">
                    <h3 className="text-xl font-semibold text-shade-primary">
                      Deposit BTC, Get USDC Instantly
                    </h3>
                    <Icon
                      name="Provide"
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
                      Receive zBTC Tokens
                    </h3>
                    <Icon
                      name="zbtc"
                      size={18}
                      className="text-primary-apollo"
                    />
                  </div>
                  <p className="text-shade-secondary">
                    Get zBTC tokens representing your deposited BTC plus
                    interest. These tokens can be held or transferred.
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
                  4
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
                    Get your BTC back at the end of the term by returning your
                    zBTC tokens. Early withdrawal options available.
                  </p>
                </div>
              </div>
            </div>
          </div>

          {/* Dashboard 및 이자 정보 */}
          <div className="grid grid-cols-1 items-center gap-12 md:grid-cols-2">
            <div>
              <h3 className="mb-6 text-2xl font-bold text-shade-primary">
                Interest Claims & Dashboard
              </h3>
              <p className="mb-6 text-lg text-shade-secondary">
                Claim your interest anytime and monitor your assets on the
                dashboard after deposit.
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
                  Example: BTC Deposit
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
                <div className="flex items-center justify-between">
                  <span className="text-shade-secondary">Issued Token</span>
                  <span className="font-medium">1 zBTC</span>
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

      {/* CTA 섹션 */}
      <section className="mx-4 my-12 rounded-3xl bg-gradient-to-r from-primary-apollo/10 to-primary-apollo/20 px-4 py-20 md:mx-8">
        <div className="mx-auto max-w-5xl text-center">
          <h2 className="mb-6 text-3xl font-bold text-shade-primary md:text-4xl">
            Deposit BTC and Earn Interest Now
          </h2>
          <p className="mx-auto mb-10 max-w-3xl text-lg text-shade-secondary">
            Don't wait any longer. Deposit now and claim your interest
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

// 단계 카드 컴포넌트 타입 정의
interface StepCardProps {
  number: number;
  title: string;
  description: string;
  iconName: IconName;
}

// 단계 카드 컴포넌트
function StepCard({ number, title, description, iconName }: StepCardProps) {
  return (
    <div className="rounded-2xl border border-primary-apollo/10 bg-white p-6 shadow-sm transition-shadow duration-300 hover:shadow-md">
      <div className="flex items-start gap-4">
        <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-primary-apollo font-bold text-white">
          {number}
        </div>
        <div className="flex-1">
          <div className="mb-2 flex items-center gap-2">
            <h3 className="text-xl font-semibold text-shade-primary">
              {title}
            </h3>
            <Icon name={iconName} size={18} className="text-primary-apollo" />
          </div>
          <p className="text-shade-secondary">{description}</p>
        </div>
      </div>
    </div>
  );
}
