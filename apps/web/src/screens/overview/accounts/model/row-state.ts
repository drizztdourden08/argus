/* @layer renderer-app @kind logic */
import type { AccountsSettingsApi } from '../../../../data/accounts';
import { limitTip } from '../accounts.constants';
import type { Account, RowState } from '../accounts.type';
import { canReceive } from './can-receive';
import { resetClock } from './reset-clock';
import { weeklyLimit } from './weekly-limit';

const rowState = (account: Account, rules: AccountsSettingsApi, now: number, switching: boolean): RowState => {
  const receive = canReceive(account, rules);
  const limit = weeklyLimit(account);
  const until = resetClock(limit?.resetsAt, now);
  const full = limit !== null;
  const blocked = account.active === true || switching || !receive || full;
  return { receive, full, until, blocked, tip: full ? limitTip(until) : undefined };
};

export { rowState };
