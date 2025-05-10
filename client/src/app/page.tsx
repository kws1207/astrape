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
            예치한 BTC에 대한 선이자를 USDC로 즉시 받아가세요. 안전하고 투명한
            방식으로 암호화폐 자산을 성장시킬 수 있습니다.
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
            주요 특징
          </h2>
          <div className="grid grid-cols-1 gap-8 md:grid-cols-3">
            <FeatureCard
              title="선이자 지급"
              description="BTC 예치와 동시에 zBTC 발행 및 USDC로 선이자를 즉시 지급해 드립니다."
              iconName="Provide"
            />
            <FeatureCard
              title="직관적인 UX"
              description="복잡한 과정 없이 간편하게 자산을 예치하고 이자를 얻을 수 있습니다."
              iconName="Interaction"
            />
            <FeatureCard
              title="지속 가능한 일드"
              description="이자풀 운영을 통해 안정적이고 지속 가능한 수익을 제공합니다."
              iconName="Portfolio"
            />
          </div>
        </div>
      </section>

      {/* 사용 방법 */}
      <section className="px-4 py-16">
        <div className="mx-auto max-w-7xl">
          <h2 className="mb-4 text-center text-3xl font-bold text-shade-primary">
            사용 방법
          </h2>
          <p className="mx-auto mb-12 max-w-3xl text-center text-shade-secondary">
            간단한 몇 단계만으로 BTC를 예치하고 이자를 받을 수 있습니다.
          </p>
          <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
            <StepCard
              number={1}
              title="지갑 연결"
              description="암호화폐 지갑을 연결하여 시작하세요."
              iconName="Wallet"
            />
            <StepCard
              number={2}
              title="예치 안내"
              description="예치 과정과 이자 정보를 확인하세요."
              iconName="Info"
            />
            <StepCard
              number={3}
              title="금액 및 기간 선택"
              description="원하는 BTC 양과 예치 기간을 선택하세요."
              iconName="btc"
            />
            <StepCard
              number={4}
              title="이자 미리보기"
              description="받게 될 이자를 미리 확인하세요."
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
              이자 클레임 및 대시보드
            </h2>
            <p className="mb-6 text-lg text-shade-secondary">
              예치 후 언제든지 이자를 클레임하고 대시보드에서 자산 현황을 확인할
              수 있습니다.
            </p>
            <ul className="space-y-4">
              <li className="flex items-start">
                <Icon
                  name="Tick"
                  size={18}
                  className="mr-2 mt-1 text-primary-apollo"
                />
                <span>언제든지 클레임 가능한 이자 확인</span>
              </li>
              <li className="flex items-start">
                <Icon
                  name="Tick"
                  size={18}
                  className="mr-2 mt-1 text-primary-apollo"
                />
                <span>실시간 자산 가치 모니터링</span>
              </li>
              <li className="flex items-start">
                <Icon
                  name="Tick"
                  size={18}
                  className="mr-2 mt-1 text-primary-apollo"
                />
                <span>예치 만기일 및 해지 옵션 관리</span>
              </li>
            </ul>
          </div>
          <div className="rounded-2xl border border-primary-apollo/10 bg-white p-6 shadow-lg">
            <div className="mb-4 rounded-xl bg-gradient-to-r from-primary-apollo/10 to-primary-apollo/5 p-4">
              <h3 className="mb-2 text-xl font-semibold text-shade-primary">
                예시: BTC 예치
              </h3>
              <div className="mb-2 flex items-center justify-between">
                <span className="text-shade-secondary">예치 금액</span>
                <span className="font-medium">1 BTC</span>
              </div>
              <div className="mb-2 flex items-center justify-between">
                <span className="text-shade-secondary">예치 기간</span>
                <span className="font-medium">12개월</span>
              </div>
              <div className="mb-2 flex items-center justify-between">
                <span className="text-shade-secondary">선이자 지급</span>
                <span className="font-medium text-green-600">+500 USDC</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-shade-secondary">발행 토큰</span>
                <span className="font-medium">1 zBTC</span>
              </div>
            </div>
            <div className="rounded-xl bg-primary-apollo/5 p-4">
              <div className="text-center">
                <span className="mb-1 block text-shade-secondary">
                  연 이자율
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
            지금 바로 BTC를 예치하고 이자를 받아보세요
          </h2>
          <p className="mx-auto mb-10 max-w-3xl text-lg text-shade-secondary">
            더 이상 기다릴 필요가 없습니다. 지금 예치하고 즉시 이자를
            받아가세요.
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
