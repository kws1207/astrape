"use client";

import { motion, AnimatePresence } from "framer-motion";
import Icon from "@/components/Icons";
import { useCallback } from "react";
import { useRouter } from "next/navigation";

type DepositSuccessModalProps = {
  isOpen: boolean;
  onClose: () => void;
  transactionData?: {
    depositAmount: number;
    depositPeriod: string;
    receiveAmount: number;
    transactionSignature?: string;
  };
};

export function DepositSuccessModal({
  isOpen,
  onClose,
  transactionData,
}: DepositSuccessModalProps) {
  const router = useRouter();

  const handleClose = useCallback(() => {
    onClose();
    router.push("/dashboard");
  }, [onClose, router]);

  const handleBackdropClick = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      if (e.target === e.currentTarget) {
        handleClose();
      }
    },
    [handleClose]
  );

  const getPeriodText = (period: string) => {
    const months = Number(period.replace("M", ""));
    return `${months} ${months === 1 ? "month" : "months"}`;
  };

  const viewTransaction = useCallback(() => {
    if (transactionData?.transactionSignature) {
      window.open(
        `https://explorer.solana.com/tx/${transactionData.transactionSignature}`,
        "_blank"
      );
    }
  }, [transactionData?.transactionSignature]);

  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={handleBackdropClick}
        >
          <motion.div
            className="w-full max-w-md rounded-2xl border border-primary-apollo/10 bg-white p-6 shadow-xl"
            initial={{ scale: 0.9, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            exit={{ scale: 0.9, opacity: 0 }}
            transition={{ type: "spring", damping: 25, stiffness: 300 }}
          >
            <div className="mb-6 flex justify-between">
              <h2 className="text-2xl font-bold text-shade-primary">
                Deposit Successful
              </h2>
              <button
                onClick={handleClose}
                className="rounded-full p-1 text-shade-secondary transition-colors hover:bg-gray-100 hover:text-shade-primary"
              >
                <Icon name="Close" size={18} />
              </button>
            </div>

            {transactionData && (
              <motion.div
                className="mb-6 rounded-xl bg-gradient-to-r from-green-50 to-green-100 p-6 text-center"
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.3 }}
              >
                <p className="mb-2 text-sm font-medium text-green-600">
                  You&apos;ve received
                </p>
                <p className="mb-1 text-3xl font-bold text-shade-primary">
                  ${transactionData.receiveAmount.toLocaleString()}
                </p>
                <p className="text-lg font-medium text-green-600">USDC</p>
              </motion.div>
            )}

            <div className="mb-6 flex justify-center">
              <div className="flex h-16 w-16 items-center justify-center rounded-full bg-green-100">
                <motion.div
                  initial={{ scale: 0 }}
                  animate={{ scale: 1 }}
                  transition={{
                    type: "spring",
                    damping: 10,
                    stiffness: 100,
                    delay: 0.2,
                  }}
                >
                  <Icon name="Success" size={18} className="text-green-600" />
                </motion.div>
              </div>
            </div>

            {transactionData && (
              <div className="mb-6 space-y-4 rounded-xl bg-gradient-to-r from-primary-apollo/10 to-primary-apollo/5 p-4">
                <div className="flex justify-between">
                  <span className="text-shade-secondary">You deposited</span>
                  <span className="font-medium text-shade-primary">
                    {transactionData.depositAmount} zBTC
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-shade-secondary">Lock period</span>
                  <span className="font-medium text-shade-primary">
                    {getPeriodText(transactionData.depositPeriod)}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-shade-secondary">You received</span>
                  <span className="font-medium text-shade-primary">
                    ${transactionData.receiveAmount.toLocaleString()} USDC
                  </span>
                </div>
              </div>
            )}

            <div className="flex flex-col gap-3">
              {transactionData?.transactionSignature && (
                <button
                  onClick={viewTransaction}
                  className="w-full rounded-xl border border-primary-apollo py-3 text-primary-apollo transition-all hover:bg-primary-apollo/5"
                >
                  View Transaction
                </button>
              )}
              <button
                onClick={handleClose}
                className="w-full rounded-xl bg-primary-apollo py-3 text-white transition-all hover:bg-primary-apollo/90"
              >
                Go to Dashboard
              </button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

export default DepositSuccessModal;
