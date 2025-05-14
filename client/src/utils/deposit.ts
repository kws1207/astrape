// Utility functions related to deposit calculations

/**
 * Calculate optimal APY based on the deposit amount in USD.
 * The APY follows a two-tier structure:
 *  – Up to $10M receives 21.32 %
 *  – The remainder receives 14.32 %
 */
export function calculateOptimalAPY(usdAmount: number): number {
  if (usdAmount <= 10_000_000) {
    return 0.2132;
  }

  const wfragAmount = 10_000_000;
  const fragAmount = usdAmount - wfragAmount;
  return (wfragAmount * 0.2132 + fragAmount * 0.1432) / usdAmount;
}

/**
 * Compute the minimum risk-buffer (0-1) so that the conservative APY (after commission)
 * equals the 3 % worst-case APY. This guarantees the displayed "Target APY" falls back
 * to exactly 3 % when the slider is at its maximum.
 */
export function calculateMinRiskBuffer(
  currentAPY: number,
  {
    worstCaseAPY = 0.03,
    commissionRate = 0.2,
  }: {
    worstCaseAPY?: number;
    commissionRate?: number;
  } = {}
): number {
  const effectiveCurrent = currentAPY * (1 - commissionRate);
  if (effectiveCurrent <= 0) return 1;
  const buffer = 1 - worstCaseAPY / effectiveCurrent;
  return Math.max(0, Math.min(1, buffer));
}
