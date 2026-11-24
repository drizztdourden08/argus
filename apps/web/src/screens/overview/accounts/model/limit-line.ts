/* @layer renderer-app @kind logic */
import type { AccountDoc } from '../../../../data/types';
import { atLimitText } from '../accounts.constants';
import { clockText } from './clock-text';

const limitLine = (account: AccountDoc): string => {
  if (account.limit?.confirmed !== true) return '';
  return atLimitText(clockText(account.limit.untilMs));
};

export { limitLine };
