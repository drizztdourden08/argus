/* @layer renderer-app @kind logic */
import type { AccountDoc } from '../../../../data/types';

const weeklyPercent = (account: AccountDoc | undefined): number | null => {
  const weekly = (account?.limits ?? []).find((limit) => limit.kind === 'weekly_all');
  return typeof weekly?.percent === 'number' ? Math.round(weekly.percent) : null;
};

export { weeklyPercent };
