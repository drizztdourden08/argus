/* @layer renderer-app @kind logic */
import type { AccountDoc, AccountLimit } from '../../../../data/types';

const reportedLimits = (account: AccountDoc): AccountLimit[] => (account.limits ?? []).filter((limit) => typeof limit.percent === 'number');

export { reportedLimits };
