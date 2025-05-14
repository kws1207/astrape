import { DepositPeriod } from "@/types/deposit";

export const depositPeriodsDisplayText: Record<DepositPeriod, string> = {
  "1M": "1 Month",
  "3M": "3 Months",
  "6M": "6 Months",
};

export const slotCountMap: Record<DepositPeriod, number> = {
  "1M": 5_890_909,
  "3M": 17_672_727,
  "6M": 35_345_454,
};
