"use client";

import { useAstrape } from "@/hooks/astrape/useAstrape";
import { motion } from "framer-motion";
import { UserDepositState } from "@/types/astrape";
import Icon from "@/components/Icons";
import Button from "@/components/Button/Button";
import { useRouter } from "next/navigation";
import { useWallet } from "@solana/wallet-adapter-react";
import { useEffect } from "react";

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
      return "text-green-600";
    case UserDepositState.WithdrawRequested:
      return "text-yellow-600";
    case UserDepositState.WithdrawReady:
      return "text-primary-apollo";
    default:
      return "text-gray-600";
  }
};

const getRemainingTime = (unlockSlot: number, depositSlot: number) => {
  const currentSlot =
    depositSlot + Math.floor((Date.now() / 1000 - depositSlot) * 0.4);
  const remainingSlots = Math.max(0, unlockSlot - currentSlot);
  const remainingDays = Math.ceil(remainingSlots / (2 * 60 * 24));

  return `${remainingDays} days`;
};

export default function DashboardPage() {
  const { userDeposit, mutateUserDeposit } = useAstrape();
  const router = useRouter();
  const { connected: solanaWalletConnected } = useWallet();

  useEffect(() => {
    mutateUserDeposit();
  }, [solanaWalletConnected, mutateUserDeposit]);

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
              <div className="overflow-hidden rounded-2xl border border-primary-apollo/10 bg-white shadow-lg">
                <div className="bg-gradient-to-r from-primary-apollo/10 to-primary-apollo/5 p-6">
                  <div className="mb-2 flex items-center justify-between">
                    <h2 className="text-lg font-semibold text-shade-primary md:text-xl">
                      Deposit Details
                    </h2>
                    <span
                      className={`inline-flex items-center rounded-full px-3 py-1 text-sm font-medium ${getStateColor(userDeposit.state)} bg-current bg-opacity-10`}
                    >
                      {getStateLabel(userDeposit.state)}
                    </span>
                  </div>

                  <div className="mt-4">
                    <div className="flex flex-col items-start justify-between gap-2 md:flex-row md:items-center">
                      <div>
                        <span className="text-sm text-shade-secondary">
                          Amount Deposited
                        </span>
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

                <div className="p-6">
                  <div className="grid gap-6 md:grid-cols-2">
                    <div className="space-y-4">
                      <div>
                        <div className="mb-1 flex items-center gap-2">
                          <Icon
                            name="Clock"
                            size={14}
                            className="text-primary-apollo"
                          />
                          <span className="text-sm font-medium text-shade-secondary">
                            Deposit Date
                          </span>
                        </div>
                        <p className="text-base font-medium text-shade-primary">
                          {new Date(
                            userDeposit.depositSlot * 1000
                          ).toLocaleDateString(undefined, {
                            year: "numeric",
                            month: "long",
                            day: "numeric",
                          })}
                        </p>
                      </div>

                      <div>
                        <div className="mb-1 flex items-center gap-2">
                          <Icon
                            name="Clock"
                            size={14}
                            className="text-primary-apollo"
                          />
                          <span className="text-sm font-medium text-shade-secondary">
                            Unlock Date
                          </span>
                        </div>
                        <p className="text-base font-medium text-shade-primary">
                          {new Date(
                            userDeposit.unlockSlot * 1000
                          ).toLocaleDateString(undefined, {
                            year: "numeric",
                            month: "long",
                            day: "numeric",
                          })}
                        </p>
                      </div>
                    </div>

                    <div className="space-y-4">
                      <div>
                        <div className="mb-1 flex items-center gap-2">
                          <Icon
                            name="Withdraw02"
                            size={14}
                            className="text-primary-apollo"
                          />
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
                            <Icon
                              name="Clock"
                              size={14}
                              className="text-primary-apollo"
                            />
                            <span className="text-sm font-medium text-shade-secondary">
                              Time until unlock
                            </span>
                          </div>
                          <span className="text-sm font-medium text-primary-apollo">
                            {getRemainingTime(
                              userDeposit.unlockSlot,
                              userDeposit.depositSlot
                            )}
                          </span>
                        </div>
                        <div className="mt-2 h-2 w-full overflow-hidden rounded-full bg-gray-100">
                          <div
                            className="h-full rounded-full bg-primary-apollo transition-all duration-1000"
                            style={{
                              width: `${Math.min(
                                100,
                                ((Date.now() / 1000 - userDeposit.depositSlot) /
                                  (userDeposit.unlockSlot -
                                    userDeposit.depositSlot)) *
                                  100
                              )}%`,
                            }}
                          ></div>
                        </div>
                      </div>
                    </div>
                  </div>

                  <div className="mt-8 flex flex-wrap items-center gap-4">
                    {userDeposit.state === UserDepositState.Deposited && (
                      <Button
                        label="Request Withdrawal"
                        type="secondary"
                        size="medium"
                      />
                    )}

                    {userDeposit.state === UserDepositState.WithdrawReady && (
                      <Button
                        label="Withdraw Now"
                        type="primary"
                        size="medium"
                      />
                    )}
                  </div>
                </div>
              </div>
            </div>

            <div>
              <div className="rounded-2xl border border-primary-apollo/10 bg-white p-6 shadow-lg">
                <h2 className="mb-4 text-lg font-semibold text-shade-primary">
                  Summary
                </h2>

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
                      className={`font-medium ${getStateColor(userDeposit.state)}`}
                    >
                      {getStateLabel(userDeposit.state)}
                    </span>
                  </div>

                  <div className="mt-6 rounded-xl bg-primary-apollo/5 p-4 text-center">
                    <span className="block text-sm text-shade-secondary">
                      Total Value
                    </span>
                    <span className="text-2xl font-bold text-shade-primary">
                      {userDeposit.amount.toLocaleString()} zBTC +{" "}
                      {userDeposit.interestReceived.toLocaleString()} USDC
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        ) : (
          <div className="flex min-h-[300px] items-center justify-center rounded-2xl border border-primary-apollo/10 bg-white p-8 shadow-lg">
            <div className="max-w-md text-center">
              <div className="mb-6 flex justify-center">
                <div className="rounded-full bg-primary-apollo/10 p-4">
                  <Icon
                    name="Portfolio"
                    size={18}
                    className="text-primary-apollo"
                  />
                </div>
              </div>
              <h2 className="mb-3 text-xl font-semibold text-shade-primary">
                No Deposits Found
              </h2>
              <p className="mb-8 text-shade-secondary">
                You haven&apos;t made any deposits yet. Start earning by making
                your first deposit now.
              </p>
              <div className="flex justify-center">
                <Button
                  label="Make Your First Deposit"
                  type="primary"
                  size="medium"
                  onClick={() => router.push("/deposit")}
                />
              </div>
            </div>
          </div>
        )}
      </motion.div>
    </main>
  );
}
