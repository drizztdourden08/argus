/* @layer renderer-app @kind logic */
import type { AccountDoc, AccountLimit } from '../../../../data/types';
import { MAX_PERCENT, WEEKLY_KIND } from '../accounts.constants';
import { usageOf } from './usage-of';

const weeklyLimit = (account: AccountDoc): AccountLimit | null => {
  const limit = usageOf(account, WEEKLY_KIND);
  const full = typeof limit?.percent === 'number' && limit.percent >= MAX_PERCENT;
  return full && limit.isActive !== false ? limit : null;
};

export { weeklyLimit };
