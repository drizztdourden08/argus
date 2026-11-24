/* @layer renderer-app @kind types */
import type { AccountsSettingsApi } from '../../../data/accounts';
import type { AccountLimit, AccountsDrift, AccountsRun } from '../../../data/types';
import type { Account, AccountActions, RowState } from './accounts.type';
import type { SyncStep } from './compare.type';

interface AccountNameProps {
  account: Account;
}

interface LimitMeterProps {
  limit: AccountLimit;
  now: number;
}

interface LimitMetersProps {
  account: Account;
  now: number;
}

interface AccountFactsProps {
  account: Account;
  differs: string | null;
}

interface SwitchActionProps {
  account: Account;
  current: Account | null;
  state: RowState;
  switching: boolean;
  confirming: boolean;
  onSwitch: (key: string) => void;
}

interface RuleFieldsProps {
  accountKey: string;
  rules: AccountsSettingsApi;
}

interface SyncCardProps {
  drift: AccountsDrift | null;
  accounts: readonly Account[];
  run: AccountsRun | null;
  steps: readonly SyncStep[];
  actions: AccountActions;
  onReview: () => void;
}

interface SyncFooterProps {
  drift: AccountsDrift | null;
  run: AccountsRun | null;
  active: boolean;
}

interface SyncActionsProps {
  run: AccountsRun | null;
  actions: AccountActions;
  onReview: () => void;
}

interface SyncRunLineProps {
  run: AccountsRun;
  steps: readonly SyncStep[];
  accounts: readonly Account[];
}

interface AccountsOverviewProps {
  accounts: readonly Account[];
  drift: AccountsDrift | null;
  onReview: () => void;
}

export type {
  AccountFactsProps, AccountNameProps, AccountsOverviewProps, LimitMeterProps, LimitMetersProps, RuleFieldsProps, SwitchActionProps, SyncActionsProps, SyncFooterProps,
  SyncCardProps, SyncRunLineProps,
};
