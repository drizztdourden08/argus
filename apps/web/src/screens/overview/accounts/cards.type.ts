/* @layer renderer-app @kind types */
import type { AccountsSettingsApi } from '../../../data/accounts';
import type { AccountLimit, AccountsDrift, AccountsRun, AccountsSwitch } from '../../../data/types';
import type { Account, AccountActions, RowState } from './accounts.type';
import type { CompareDoc, SyncStep } from './compare.type';

interface AccountNameProps {
  account: Account;
}

interface AccountCardsProps {
  accounts: readonly Account[];
  rules: AccountsSettingsApi;
  compare: CompareDoc | null;
  confirmKey: string | null;
  switching: boolean;
  onSwitch: (key: string) => void;
}

interface AccountCardProps {
  account: Account;
  current: Account | null;
  rules: AccountsSettingsApi;
  compare: CompareDoc | null;
  now: number;
  confirming: boolean;
  switching: boolean;
  onSwitch: (key: string) => void;
}

interface ReceiveLineProps {
  state: RowState;
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

interface RulesCardProps {
  accounts: readonly Account[];
  rules: AccountsSettingsApi;
}

interface RulesBlockProps {
  account: Account;
  rules: AccountsSettingsApi;
}

interface RuleFieldsProps {
  accountKey: string;
  rules: AccountsSettingsApi;
}

interface SyncCardProps {
  drift: AccountsDrift | null;
  accounts: readonly Account[];
  run: AccountsRun | null;
  actions: AccountActions;
  onReview: () => void;
}

interface SyncActionsProps {
  run: AccountsRun | null;
  actions: AccountActions;
  onReview: () => void;
}

interface SyncChoicesProps {
  accounts: readonly Account[];
  disabled: boolean;
  actions: AccountActions;
}

interface SyncRunLineProps {
  run: AccountsRun;
  steps: readonly SyncStep[];
  accounts: readonly Account[];
}

interface ActivityCardProps {
  doc: AccountsSwitch | null;
  accounts: readonly Account[];
  confirmKey: string | null;
  drifted: boolean;
  run: AccountsRun | null;
  steps: readonly SyncStep[];
  onCloseConfirm: () => void;
}

interface AccountsOverviewProps {
  accounts: readonly Account[];
  drift: AccountsDrift | null;
  onReview: () => void;
}

export type {
  AccountCardProps, AccountCardsProps, AccountFactsProps, AccountNameProps, AccountsOverviewProps, ActivityCardProps, LimitMeterProps,
  LimitMetersProps, ReceiveLineProps, RuleFieldsProps, RulesBlockProps, RulesCardProps, SwitchActionProps, SyncActionsProps, SyncCardProps,
  SyncChoicesProps, SyncRunLineProps,
};
