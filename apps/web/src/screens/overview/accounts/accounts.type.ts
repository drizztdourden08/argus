/* @layer renderer-app @kind types */
import type { AccountsSettingsApi } from '../../../data/accounts';
import type { WithId } from '../../../data/react';
import type { AccountDoc, AccountLimit, AccountsDrift, AccountsRun } from '../../../data/types';
import type { SyncStep } from './compare.type';

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

type AccountsView = 'overview' | 'review';

interface AccountsStripProps {
  accounts: readonly Account[];
  rules: AccountsSettingsApi;
  confirmKey: string | null;
  switching: boolean;
  onSwitch: (key: string) => void;
  onCloseConfirm: () => void;
}

interface AccountRowProps {
  account: Account;
  rules: AccountsSettingsApi;
  now: number;
  confirming: boolean;
  switching: boolean;
  onSwitch: (key: string) => void;
  onCloseConfirm: () => void;
}

interface AccountNameProps {
  account: Account;
}

interface UsageCellProps {
  limit: AccountLimit | null;
  label: string;
  now: number;
}

interface SyncSectionProps {
  drift: AccountsDrift | null;
  accounts: readonly Account[];
  actions: AccountActions;
  onReview: () => void;
}

interface SyncRunLineProps {
  run: AccountsRun;
  steps: readonly SyncStep[];
  accounts: readonly Account[];
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

export type {
  Account, AccountActions, AccountNameProps, AccountRowProps, AccountsFeed, AccountsStripProps, AccountsView, PercentFieldProps, RuleChipsProps,
  SyncRunLineProps, SyncSectionProps, UsageCellProps,
};
