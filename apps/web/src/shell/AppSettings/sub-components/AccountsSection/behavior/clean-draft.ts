/* @layer renderer-app @kind logic */
import type { AccountsSettings, HandoverRule, SettingsAccount } from '../../../../../data/types';
import { DEFAULT_HANDOVER_PERCENT, DEFAULT_SWITCH_AT } from '../../../../../screens/overview/accounts/accounts.constants';
import { clampPercent } from '../../../../../screens/overview/accounts/model';

const handoverOf = (next: HandoverRule, was: HandoverRule | undefined): HandoverRule => {
  if (typeof next !== 'number') return next;
  return clampPercent(next, typeof was === 'number' ? was : DEFAULT_HANDOVER_PERCENT);
};

const cleanAccount = (item: SettingsAccount, saved: SettingsAccount | undefined): SettingsAccount => ({
  ...item,
  rules: {
    ...item.rules,
    switchAt: clampPercent(item.rules.switchAt, saved?.rules.switchAt ?? DEFAULT_SWITCH_AT),
    handover: handoverOf(item.rules.handover, saved?.rules.handover),
  },
});

const cleanDraft = (draft: AccountsSettings, saved: AccountsSettings | null): AccountsSettings => ({
  ...draft,
  accounts: draft.accounts.map((item) => cleanAccount(item, saved?.accounts.find((was) => was.key === item.key))),
});

export { cleanDraft };
