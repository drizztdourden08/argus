/* @layer renderer-app @kind logic */
import type { AccountsSettingsApi } from '../../../../data/accounts';
import type { Account } from '../accounts.type';

const canReceive = (account: Account, rules: AccountsSettingsApi): boolean => {
  const mine = rules.settings?.accounts.find((item) => item.key === (account.key ?? account.id));
  return mine ? mine.canReceive : account.rules?.canReceive !== false;
};

export { canReceive };
