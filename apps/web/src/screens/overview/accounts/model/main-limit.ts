/* @layer renderer-app @kind logic */
import type { AccountDoc, AccountLimit } from '../../../../data/types';
import { reportedLimits } from './reported-limits';

const mainLimit = (account: AccountDoc): AccountLimit | null =>
  reportedLimits(account).reduce<AccountLimit | null>((top, limit) => (top === null || (limit.percent ?? 0) > (top.percent ?? 0) ? limit : top), null);

export { mainLimit };
