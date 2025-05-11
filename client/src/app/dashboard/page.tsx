"use client";

import { useAstrape } from "@/hooks/astrape/useAstrape";
import { motion } from "framer-motion";
import { UserDepositState, UserDeposit } from "@/types/astrape";
import Icon from "@/components/Icons";
import Button from "@/components/Button/Button";
import { useRouter } from "next/navigation";
import { useWallet } from "@solana/wallet-adapter-react";
import { useEffect, useState } from "react";

const getStateLabel = (state: UserDepositState) => {
  switch (state) {
    case UserDepositState.Deposited:
      return "Active";
    case UserDepositState.WithdrawRequested:
      return "Withdrawal Requested";
    case UserDepositState.WithdrawReady:
      return "Ready to Withdraw";
    default:
      return "Unknown";
  }
};

const getStateColor = (state: UserDepositState) => {
  switch (state) {
    case UserDepositState.Deposited:
      return "green-600";
    case UserDepositState.WithdrawRequested:
      return "yellow-600";
    case UserDepositState.WithdrawReady:
      return "primary-apollo";
    default:
      return "gray-600";
  }
};

const getRemainingTime = (unlockSlot: number, depositSlot: number) => {
  const currentSlot =
    depositSlot + Math.floor((Date.now() / 1000 - depositSlot) * 0.4);
  const remainingSlots = Math.max(0, unlockSlot - currentSlot);
  const remainingDays = Math.ceil(remainingSlots / (2 * 60 * 24));

  return `${remainingDays} days`;
};

const isBeforeUnlock = (unlockSlot: number, depositSlot: number) => {
  const currentSlot =
    depositSlot + Math.floor((Date.now() / 1000 - depositSlot) * 0.4);
  return currentSlot < unlockSlot;
};

const EmptyDepositState = ({
  onClickDeposit,
}: {
  onClickDeposit: () => void;
}) => (
  <div className="flex min-h-[300px] items-center justify-center rounded-2xl border border-primary-apollo/10 bg-white p-8 shadow-lg">
    <div className="max-w-md text-center">
      <div className="mb-6 flex justify-center">
        <div className="rounded-full bg-primary-apollo/10 p-4">
          <Icon name="Portfolio" size={18} className="text-primary-apollo" />
        </div>
      </div>
      <h2 className="mb-3 text-xl font-semibold text-shade-primary">
        No Deposits Found
      </h2>
      <p className="mb-8 text-shade-secondary">
        You haven&apos;t made any deposits yet. Start earning by making your
        first deposit now.
      </p>
      <div className="flex justify-center">
        <Button
          label="Make Your First Deposit"
          type="primary"
          size="medium"
          onClick={onClickDeposit}
        />
      </div>
    </div>
  </div>
);

// DepositSummary component
const DepositSummary = ({ userDeposit }: { userDeposit: UserDeposit }) => (
  <div className="rounded-2xl border border-primary-apollo/10 bg-white p-6 shadow-lg">
    <h2 className="mb-4 text-lg font-semibold text-shade-primary">Summary</h2>

    <div className="space-y-4">
      <div className="flex justify-between border-b border-primary-apollo/5 pb-3">
        <span className="text-shade-secondary">Principal</span>
        <span className="font-medium text-shade-primary">
          {userDeposit.amount.toLocaleString()} zBTC
        </span>
      </div>

      <div className="flex justify-between border-b border-primary-apollo/5 pb-3">
        <span className="text-shade-secondary">Interest</span>
        <span className="font-medium text-green-600">
          {userDeposit.interestReceived.toLocaleString()} USDC
        </span>
      </div>

      <div className="flex justify-between border-b border-primary-apollo/5 pb-3">
        <span className="text-shade-secondary">Commission</span>
        <span className="font-medium text-shade-primary">
          {(userDeposit.commissionRate / 10000).toFixed(2)}%
        </span>
      </div>

      <div className="flex justify-between border-b border-primary-apollo/5 pb-3">
        <span className="text-shade-secondary">Status</span>
        <span
          className={`font-medium text-${getStateColor(userDeposit.state)}`}
        >
          {getStateLabel(userDeposit.state)}
        </span>
      </div>

      <div className="mt-6 rounded-xl bg-primary-apollo/5 p-4 text-center">
        <span className="block text-sm text-shade-secondary">Total Value</span>
        <span className="text-2xl font-bold text-shade-primary">
          {userDeposit.amount.toLocaleString()} zBTC +{" "}
          {userDeposit.interestReceived.toLocaleString()} USDC
        </span>
      </div>
    </div>
  </div>
);

const DepositDetails = ({ userDeposit }: { userDeposit: UserDeposit }) => (
  <div className="bg-gradient-to-r from-primary-apollo/10 to-primary-apollo/5 p-6">
    <div className="mb-2 flex items-center justify-between">
      <h2 className="text-lg font-semibold text-shade-primary md:text-xl">
        Deposit Details
      </h2>
      <span
        className={`inline-flex items-center rounded-full px-3 py-1 text-sm font-medium bg-${getStateColor(userDeposit.state)} text-white`}
      >
        {getStateLabel(userDeposit.state)}
      </span>
    </div>

    <div className="mt-4">
      <div className="flex flex-col items-start justify-between gap-2 md:flex-row md:items-center">
        <div>
          <span className="text-sm text-shade-secondary">Amount Deposited</span>
          <p className="text-xl font-bold text-shade-primary">
            {userDeposit.amount.toLocaleString()} zBTC
          </p>
        </div>
        <div className="hidden h-10 w-px bg-primary-apollo/10 md:block"></div>
        <div>
          <span className="text-sm text-shade-secondary">
            Interest Received
          </span>
          <p className="text-xl font-bold text-green-600">
            {userDeposit.interestReceived.toLocaleString()} USDC
          </p>
        </div>
      </div>
    </div>
  </div>
);

const DepositTimeline = ({ userDeposit }: { userDeposit: UserDeposit }) => (
  <div className="grid gap-6 md:grid-cols-2">
    <div className="space-y-4">
      <div>
        <div className="mb-1 flex items-center gap-2">
          <Icon name="Clock" size={14} className="text-primary-apollo" />
          <span className="text-sm font-medium text-shade-secondary">
            Deposit Date
          </span>
        </div>
        <p className="text-base font-medium text-shade-primary">
          {new Date(userDeposit.depositSlot * 1000).toLocaleDateString(
            undefined,
            {
              year: "numeric",
              month: "long",
              day: "numeric",
            }
          )}
        </p>
      </div>

      <div>
        <div className="mb-1 flex items-center gap-2">
          <Icon name="Clock" size={14} className="text-primary-apollo" />
          <span className="text-sm font-medium text-shade-secondary">
            Unlock Date
          </span>
        </div>
        <p className="text-base font-medium text-shade-primary">
          {new Date(userDeposit.unlockSlot * 1000).toLocaleDateString(
            undefined,
            {
              year: "numeric",
              month: "long",
              day: "numeric",
            }
          )}
        </p>
      </div>
    </div>

    <div className="space-y-4">
      <div>
        <div className="mb-1 flex items-center gap-2">
          <Icon name="Withdraw02" size={14} className="text-primary-apollo" />
          <span className="text-sm font-medium text-shade-secondary">
            Commission Rate
          </span>
        </div>
        <p className="text-base font-medium text-shade-primary">
          {(userDeposit.commissionRate / 10000).toFixed(2)}%
        </p>
      </div>

      <div>
        <div className="mb-1 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Icon name="Clock" size={14} className="text-primary-apollo" />
            <span className="text-sm font-medium text-shade-secondary">
              Time until unlock
            </span>
          </div>
          <span className="text-sm font-medium text-primary-apollo">
            {getRemainingTime(userDeposit.unlockSlot, userDeposit.depositSlot)}
          </span>
        </div>
        <div className="mt-2 h-2 w-full overflow-hidden rounded-full bg-gray-100">
          <div
            className="h-full rounded-full bg-primary-apollo transition-all duration-1000"
            style={{
              width: `${Math.min(
                100,
                ((Date.now() / 1000 - userDeposit.depositSlot) /
                  (userDeposit.unlockSlot - userDeposit.depositSlot)) *
                  100
              )}%`,
            }}
          ></div>
        </div>
      </div>
    </div>
  </div>
);

const WithdrawDepositedState = ({
  userDeposit,
  showWithdrawalInfo,
  setShowWithdrawalInfo,
  isProcessing,
  handleRequestWithdrawal,
}: {
  userDeposit: UserDeposit;
  showWithdrawalInfo: boolean;
  setShowWithdrawalInfo: (show: boolean) => void;
  isProcessing: boolean;
  handleRequestWithdrawal: () => Promise<void>;
}) => {
  const beforeUnlock = isBeforeUnlock(
    userDeposit.unlockSlot,
    userDeposit.depositSlot
  );

  return (
    <>
      <div className="mt-6 border-t border-primary-apollo/10 pt-6">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <h3 className="mb-1 text-base font-semibold text-shade-primary">
              {beforeUnlock
                ? "Early Withdrawal Available"
                : "Withdrawal Available"}
            </h3>
            <p className="text-sm text-shade-secondary">
              {beforeUnlock
                ? "You can request an early withdrawal with an interest refund."
                : "Your deposit has reached its maturity period and is available for withdrawal."}
            </p>
          </div>
          {!showWithdrawalInfo && (
            <Button
              label={beforeUnlock ? "Withdraw Early" : "Withdraw"}
              type={beforeUnlock ? "secondary" : "primary"}
              size="medium"
              onClick={() => setShowWithdrawalInfo(true)}
              className="min-w-[140px] justify-center shadow-sm transition-all duration-200 hover:shadow-md"
            />
          )}
        </div>
      </div>

      {showWithdrawalInfo && (
        <motion.div
          className="mt-5 rounded-lg border border-primary-apollo/10 bg-gray-50 shadow-sm"
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: "auto" }}
          transition={{ duration: 0.3 }}
        >
          <div className="border-b border-primary-apollo/10 bg-gray-50 px-5 py-4">
            <h3 className="text-lg font-semibold text-shade-primary">
              {beforeUnlock
                ? "Early Withdrawal Information"
                : "Withdrawal Information"}
            </h3>
          </div>

          <div className="p-5">
            {beforeUnlock ? (
              <EarlyWithdrawalInfo
                userDeposit={userDeposit}
                isProcessing={isProcessing}
                handleRequestWithdrawal={handleRequestWithdrawal}
                setShowWithdrawalInfo={setShowWithdrawalInfo}
              />
            ) : (
              <RegularWithdrawalInfo
                userDeposit={userDeposit}
                isProcessing={isProcessing}
                handleRequestWithdrawal={handleRequestWithdrawal}
                setShowWithdrawalInfo={setShowWithdrawalInfo}
              />
            )}
          </div>
        </motion.div>
      )}
    </>
  );
};

const EarlyWithdrawalInfo = ({
  userDeposit,
  isProcessing,
  handleRequestWithdrawal,
  setShowWithdrawalInfo,
}: {
  userDeposit: UserDeposit;
  isProcessing: boolean;
  handleRequestWithdrawal: () => Promise<void>;
  setShowWithdrawalInfo: (show: boolean) => void;
}) => (
  <>
    <div className="mb-5 flex items-start gap-3 rounded-md border border-yellow-200 bg-yellow-50 p-4">
      <div className="mt-0.5 flex-shrink-0">
        <Icon name="Alert" size={18} className="text-yellow-600" />
      </div>
      <div>
        <h4 className="mb-1 font-medium text-yellow-800">
          Early Withdrawal Notice
        </h4>
        <p className="text-sm text-yellow-700">
          You are requesting to withdraw before the unlock period.{" "}
          <strong>
            This will require you to refund a portion of your USDC interest
          </strong>{" "}
          based on the remaining lock time.
        </p>
      </div>
    </div>

    <div className="mb-5 grid gap-5 md:grid-cols-2">
      <div className="space-y-3 rounded-md border border-gray-100 bg-white p-4 shadow-sm">
        <h5 className="border-b border-gray-100 pb-1 text-sm font-medium text-shade-primary">
          Transaction Details
        </h5>
        <div className="flex justify-between pt-1">
          <span className="text-sm text-shade-secondary">
            Amount to Withdraw:
          </span>
          <span className="font-medium text-shade-primary">
            {userDeposit.amount.toLocaleString()} zBTC
          </span>
        </div>
        <div className="flex justify-between">
          <span className="text-sm text-shade-secondary">
            Interest Received:
          </span>
          <span className="font-medium text-green-600">
            {userDeposit.interestReceived.toLocaleString()} USDC
          </span>
        </div>
        <div className="mt-1 flex justify-between border-t border-red-100 pt-3">
          <span className="text-sm font-medium text-red-600">
            USDC to Refund:
          </span>
          <span className="font-medium text-red-600">
            ~{Math.round(userDeposit.interestReceived * 0.5).toLocaleString()}{" "}
            USDC
          </span>
        </div>
      </div>
    </div>

    <div className="flex flex-wrap items-center justify-end gap-4">
      <Button
        label="Cancel"
        type="secondary"
        size="medium"
        onClick={() => setShowWithdrawalInfo(false)}
        disabled={isProcessing}
        className="px-6 transition-colors duration-200 hover:bg-gray-100"
      />
      <Button
        label={isProcessing ? "Processing..." : "Request Early Withdrawal"}
        type="primary"
        size="medium"
        onClick={handleRequestWithdrawal}
        disabled={isProcessing}
        className="px-6 shadow-sm transition-all duration-200 hover:shadow-md disabled:opacity-70"
      />
    </div>
  </>
);

const RegularWithdrawalInfo = ({
  userDeposit,
  isProcessing,
  handleRequestWithdrawal,
  setShowWithdrawalInfo,
}: {
  userDeposit: UserDeposit;
  isProcessing: boolean;
  handleRequestWithdrawal: () => Promise<void>;
  setShowWithdrawalInfo: (show: boolean) => void;
}) => (
  <>
    <div className="mb-5 flex items-start gap-3 rounded-md border border-green-200 bg-green-50 p-4">
      <div className="mt-0.5 flex-shrink-0">
        <Icon name="Portfolio" size={18} className="text-green-600" />
      </div>
      <div>
        <h4 className="mb-1 font-medium text-green-800">
          Ready for Withdrawal
        </h4>
        <p className="text-sm text-green-700">
          Your deposit has reached its unlock period. You can now withdraw your
          full amount without any penalties.
        </p>
      </div>
    </div>

    <div className="mb-5 space-y-3 rounded-md border border-gray-100 bg-white p-4 shadow-sm">
      <h5 className="border-b border-gray-100 pb-1 text-sm font-medium text-shade-primary">
        Transaction Details
      </h5>
      <div className="flex justify-between pt-1">
        <span className="text-sm text-shade-secondary">
          Amount to Withdraw:
        </span>
        <span className="font-medium text-shade-primary">
          {userDeposit.amount.toLocaleString()} zBTC
        </span>
      </div>
      <div className="flex justify-between">
        <span className="text-sm text-shade-secondary">Interest Earned:</span>
        <span className="font-medium text-green-600">
          {userDeposit.interestReceived.toLocaleString()} USDC
        </span>
      </div>
    </div>

    <div className="flex flex-wrap items-center justify-end gap-4">
      <Button
        label="Cancel"
        type="secondary"
        size="medium"
        onClick={() => setShowWithdrawalInfo(false)}
        disabled={isProcessing}
        className="px-6 transition-colors duration-200 hover:bg-gray-100"
      />
      <Button
        label={isProcessing ? "Processing..." : "Request Withdrawal"}
        type="primary"
        size="medium"
        onClick={handleRequestWithdrawal}
        disabled={isProcessing}
        className="px-6 shadow-sm transition-all duration-200 hover:shadow-md disabled:opacity-70"
      />
    </div>
  </>
);

const WithdrawRequestedState = () => (
  <div className="mt-6 border-t border-primary-apollo/10 pt-6">
    <div className="rounded-lg border border-yellow-200 bg-yellow-50 p-5 shadow-sm">
      <div className="flex items-start gap-3">
        <div className="mt-0.5 flex-shrink-0">
          <Icon name="Clock" size={14} className="text-yellow-600" />
        </div>
        <div className="flex-1">
          <h4 className="mb-2 font-medium text-yellow-800">
            Withdrawal Requested
          </h4>
          <p className="text-sm text-yellow-700">
            Your withdrawal request is being processed. This typically takes
            24-48 hours to complete. You&apos;ll be notified when your funds are
            ready to withdraw.
          </p>
          <div className="mt-3 flex items-center">
            <div className="h-1.5 w-full overflow-hidden rounded-full bg-yellow-200">
              <div className="h-full w-1/3 animate-pulse rounded-full bg-yellow-500"></div>
            </div>
            <span className="ml-3 text-xs font-medium text-yellow-800">
              Processing
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
);

// WithdrawReadyState component
const WithdrawReadyState = ({
  userDeposit,
  isProcessing,
  handleWithdrawCollateral,
}: {
  userDeposit: UserDeposit;
  isProcessing: boolean;
  handleWithdrawCollateral: () => Promise<void>;
}) => (
  <div className="mt-6 border-t border-primary-apollo/10 pt-6">
    <div className="rounded-lg border border-primary-apollo/10 bg-primary-apollo/5 p-5 shadow-sm">
      <div className="mb-4 flex items-start gap-3">
        <div className="mt-0.5 flex-shrink-0">
          <Icon name="Portfolio" size={18} className="text-primary-apollo" />
        </div>
        <div className="flex-1">
          <h4 className="mb-2 font-medium text-shade-primary">
            Funds Ready to Withdraw
          </h4>
          <p className="text-sm text-shade-secondary">
            Your funds are now ready to be withdrawn to your wallet. Click the
            button below to complete the withdrawal.
          </p>
        </div>
      </div>

      <div className="mb-4 space-y-3 rounded-md border border-gray-100 bg-white p-4 shadow-sm">
        <h5 className="border-b border-gray-100 pb-1 text-sm font-medium text-shade-primary">
          Transaction Details
        </h5>
        <div className="flex justify-between pt-1">
          <span className="text-sm text-shade-secondary">
            Amount to Withdraw:
          </span>
          <span className="font-medium text-shade-primary">
            {userDeposit.amount.toLocaleString()} zBTC
          </span>
        </div>
      </div>

      <div className="flex justify-end">
        <Button
          label={isProcessing ? "Processing..." : "Withdraw Now"}
          type="primary"
          size="medium"
          onClick={handleWithdrawCollateral}
          disabled={isProcessing}
          className="px-6 shadow-sm transition-all duration-200 hover:shadow-md disabled:opacity-70"
        />
      </div>
    </div>
  </div>
);

// DepositCard component combines all deposit-related components
const DepositCard = ({
  userDeposit,
  showWithdrawalInfo,
  setShowWithdrawalInfo,
  isProcessing,
  handleRequestWithdrawal,
  handleWithdrawCollateral,
}: {
  userDeposit: UserDeposit;
  showWithdrawalInfo: boolean;
  setShowWithdrawalInfo: (show: boolean) => void;
  isProcessing: boolean;
  handleRequestWithdrawal: () => Promise<void>;
  handleWithdrawCollateral: () => Promise<void>;
}) => (
  <div className="overflow-hidden rounded-2xl border border-primary-apollo/10 bg-white shadow-lg">
    <DepositDetails userDeposit={userDeposit} />

    <div className="p-6">
      <DepositTimeline userDeposit={userDeposit} />

      {userDeposit.state === UserDepositState.Deposited && (
        <WithdrawDepositedState
          userDeposit={userDeposit}
          showWithdrawalInfo={showWithdrawalInfo}
          setShowWithdrawalInfo={setShowWithdrawalInfo}
          isProcessing={isProcessing}
          handleRequestWithdrawal={handleRequestWithdrawal}
        />
      )}

      {userDeposit.state === UserDepositState.WithdrawRequested && (
        <WithdrawRequestedState />
      )}

      {userDeposit.state === UserDepositState.WithdrawReady && (
        <WithdrawReadyState
          userDeposit={userDeposit}
          isProcessing={isProcessing}
          handleWithdrawCollateral={handleWithdrawCollateral}
        />
      )}
    </div>
  </div>
);

export default function DashboardPage() {
  const {
    userDeposit,
    mutateUserDeposit,
    requestWithdrawal,
    withdrawCollateral,
  } = useAstrape();
  const router = useRouter();
  const { connected: solanaWalletConnected } = useWallet();
  const [isProcessing, setIsProcessing] = useState(false);
  const [showWithdrawalInfo, setShowWithdrawalInfo] = useState(false);

  useEffect(() => {
    mutateUserDeposit();
  }, [solanaWalletConnected, mutateUserDeposit]);

  useEffect(() => {
    if (
      userDeposit &&
      userDeposit.state === UserDepositState.Deposited &&
      isBeforeUnlock(userDeposit.unlockSlot, userDeposit.depositSlot)
    ) {
      setShowWithdrawalInfo(true);
    }
  }, [userDeposit]);

  const handleRequestWithdrawal = async () => {
    try {
      setIsProcessing(true);
      await requestWithdrawal();
      mutateUserDeposit();
      setShowWithdrawalInfo(false);
    } catch (error) {
      console.error("Failed to request withdrawal:", error);
    } finally {
      setIsProcessing(false);
    }
  };

  const handleWithdrawCollateral = async () => {
    try {
      setIsProcessing(true);
      await withdrawCollateral();
      mutateUserDeposit();
    } catch (error) {
      console.error("Failed to withdraw collateral:", error);
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <main className="page-content ds">
      <motion.div
        className="container mx-auto px-4 py-8 md:px-8 md:py-12"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.4 }}
      >
        <div className="mb-8 flex items-center justify-start pt-[36px]">
          <h1 className="text-2xl font-bold text-shade-primary md:text-3xl">
            Your Deposits
          </h1>
        </div>

        {userDeposit ? (
          <div className="grid gap-8 md:grid-cols-3">
            <div className="md:col-span-2">
              <DepositCard
                userDeposit={userDeposit}
                showWithdrawalInfo={showWithdrawalInfo}
                setShowWithdrawalInfo={setShowWithdrawalInfo}
                isProcessing={isProcessing}
                handleRequestWithdrawal={handleRequestWithdrawal}
                handleWithdrawCollateral={handleWithdrawCollateral}
              />
            </div>

            <div>
              <DepositSummary userDeposit={userDeposit} />
            </div>
          </div>
        ) : (
          <EmptyDepositState onClickDeposit={() => router.push("/deposit")} />
        )}
      </motion.div>
    </main>
  );
}
