/* @layer renderer-app @kind types */
import type { AccountsSettingsApi } from '../../../data/accounts';
import type { WithId } from '../../../data/react';
import type { AccountDoc, AccountLimit, AccountsDrift } from '../../../data/types';

type Account = WithId<AccountDoc>;

interface AccountsFeed {
  ready: boolean;
  accounts: Account[];
  drift: AccountsDrift | null;
}

interface AccountActions {
  busy: boolean;
  sync: () => void;
  undo: () => void;
}

type AccountsView = 'overview' | 'review' | 'applying';

interface AccountsSummaryProps {
  accounts: readonly Account[];
  drift: number;
}

interface SyncRowProps {
  drift: number;
  actions: AccountActions;
  onReview: () => void;
}

interface AccountCardProps {
  account: Account;
  actions: AccountActions;
  rules: AccountsSettingsApi;
  onSwitch: (key: string) => void;
}

interface LimitMeterProps {
  limit: AccountLimit;
}

interface AccountFactsProps {
  account: Account;
}

interface RuleChipsProps {
  account: Account;
  rules: AccountsSettingsApi;
}

interface PercentFieldProps {
  value: number;
  label: string;
  disabled: boolean;
  onCommit: (value: number) => void;
}

export type { Account, AccountActions, AccountCardProps, AccountFactsProps, AccountsFeed, AccountsSummaryProps, AccountsView, LimitMeterProps, PercentFieldProps, RuleChipsProps, SyncRowProps };
