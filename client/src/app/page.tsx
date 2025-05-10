"use client";

import { useRouter } from "next/navigation";
import Button from "@/components/Button/Button";
import Icon from "@/components/Icons";
import { IconName } from "@/components/Icons/icons";

export default function Home() {
  const router = useRouter();

  const handleGetStarted = () => {
    router.push("/mint");
  };

  return (
    <main className="w-full">
      {/* 히어로 섹션 */}
      <section className="relative flex flex-col items-center justify-center overflow-hidden px-4 py-24 text-center">
        {/* 배경 효과 */}
        <div className="absolute inset-0 bg-gradient-to-b from-primary-apollo/5 to-transparent" />
        <div className="absolute -right-20 top-20 h-64 w-64 rounded-full bg-primary-apollo/5 blur-3xl md:h-96 md:w-96" />
        <div className="absolute -left-20 bottom-20 h-64 w-64 rounded-full bg-primary-apollo/5 blur-3xl md:h-96 md:w-96" />

        {/* 컨텐츠 */}
        <div className="relative z-10">
          <h1 className="mb-6 text-4xl font-bold text-shade-primary md:text-6xl">
            <span className="text-primary-apollo">Deposit BTC</span> & Earn
            Interest Now
          </h1>
          <p className="mb-10 max-w-3xl text-lg text-shade-secondary md:text-xl">
            Receive immediate interest in USDC for your deposited BTC. Grow your
            crypto assets in a safe and transparent way.
          </p>
          <Button
            label="Get Started"
            type="primary"
            size="large"
            onClick={handleGetStarted}
          />
        </div>
      </section>

      {/* 주요 특징 */}
      <section className="bg-white bg-opacity-50 px-4 py-16 backdrop-blur-sm">
        <div className="mx-auto max-w-7xl">
          <h2 className="mb-12 text-center text-3xl font-bold text-shade-primary">
            Key Features
          </h2>
          <div className="grid grid-cols-1 gap-8 md:grid-cols-3">
            <FeatureCard
              title="Upfront Interest"
              description="Receive immediate USDC interest and zBTC issuance when you deposit BTC."
              iconName="Provide"
            />
            <FeatureCard
              title="Intuitive UX"
              description="Easily deposit assets and earn interest without complicated processes."
              iconName="Interaction"
            />
            <FeatureCard
              title="Sustainable Yield"
              description="Secure stable and sustainable returns through our interest pool operations."
              iconName="Portfolio"
            />
          </div>
        </div>
      </section>

      {/* 사용 방법 */}
      <section className="px-4 py-16">
        <div className="mx-auto max-w-7xl">
          <h2 className="mb-4 text-center text-3xl font-bold text-shade-primary">
            How It Works
          </h2>
          <p className="mx-auto mb-12 max-w-3xl text-center text-shade-secondary">
            Deposit BTC and earn interest in just a few simple steps.
          </p>
          <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
            <StepCard
              number={1}
              title="Connect Wallet"
              description="Start by connecting your cryptocurrency wallet."
              iconName="Wallet"
            />
            <StepCard
              number={2}
              title="Deposit Guide"
              description="Check the deposit process and interest information."
              iconName="Info"
            />
            <StepCard
              number={3}
              title="Select Amount & Period"
              description="Choose your desired BTC amount and deposit period."
              iconName="btc"
            />
            <StepCard
              number={4}
              title="Interest Preview"
              description="Preview the interest you'll receive before confirming."
              iconName="Provide"
            />
          </div>
        </div>
      </section>

      {/* 추가 정보 섹션 */}
      <section className="px-4 py-16">
        <div className="mx-auto grid max-w-7xl grid-cols-1 items-center gap-12 md:grid-cols-2">
          <div>
            <h2 className="mb-6 text-3xl font-bold text-shade-primary">
              Interest Claims & Dashboard
            </h2>
            <p className="mb-6 text-lg text-shade-secondary">
              Claim your interest anytime and monitor your assets on the
              dashboard after deposit.
            </p>
            <ul className="space-y-4">
              <li className="flex items-start">
                <Icon
                  name="Tick"
                  size={18}
                  className="mr-2 mt-1 text-primary-apollo"
                />
                <span>Check claimable interest anytime</span>
              </li>
              <li className="flex items-start">
                <Icon
                  name="Tick"
                  size={18}
                  className="mr-2 mt-1 text-primary-apollo"
                />
                <span>Monitor asset value in real-time</span>
              </li>
              <li className="flex items-start">
                <Icon
                  name="Tick"
                  size={18}
                  className="mr-2 mt-1 text-primary-apollo"
                />
                <span>Manage maturity dates and withdrawal options</span>
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
                <span className="font-medium">12 months</span>
              </div>
              <div className="mb-2 flex items-center justify-between">
                <span className="text-shade-secondary">Upfront Interest</span>
                <span className="font-medium text-green-600">+500 USDC</span>
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
                  5.0%
                </span>
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
          <Button
            label="Get Started"
            type="primary"
            size="large"
            onClick={handleGetStarted}
          />
        </div>
      </section>
    </main>
  );
}

// 특징 카드 컴포넌트 타입 정의
interface FeatureCardProps {
  title: string;
  description: string;
  iconName: IconName;
}

// 특징 카드 컴포넌트
function FeatureCard({ title, description, iconName }: FeatureCardProps) {
  return (
    <div className="rounded-2xl border border-primary-apollo/10 bg-white p-6 shadow-sm transition-shadow duration-300 hover:shadow-md">
      <div className="flex flex-col items-center text-center">
        <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-primary-apollo/10">
          <Icon name={iconName} size={18} className="text-primary-apollo" />
        </div>
        <h3 className="mb-3 text-xl font-semibold text-shade-primary">
          {title}
        </h3>
        <p className="text-shade-secondary">{description}</p>
      </div>
    </div>
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
