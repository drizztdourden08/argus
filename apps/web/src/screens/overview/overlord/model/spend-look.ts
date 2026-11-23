/* @layer renderer-app @kind logic */
import type { OverlordSpend } from '../../../../data/overlord';
import type { SpendLook } from './model.type';

const toneOf = (ratio: number, warnAt: number): SpendLook['tone'] => {
  if (ratio >= 1) return 'danger';
  return ratio >= warnAt ? 'warning' : 'success';
};

const spendLook = (spend: OverlordSpend): SpendLook | null => {
  if (spend.budgetUsd === null || spend.budgetUsd <= 0) return null;
  const ratio = spend.spentUsd / spend.budgetUsd;
  return { ratio: Math.min(ratio, 1), tone: toneOf(ratio, spend.warnAt) };
};

export { spendLook };
