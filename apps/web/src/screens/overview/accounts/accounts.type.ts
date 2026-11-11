/* @layer renderer-app @kind types */
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
  switchTo: (key: string) => void;
  sync: () => void;
  undo: () => void;
}

type AccountsView = 'overview' | 'compare' | 'applying';

interface AccountsSummaryProps {
  accounts: readonly Account[];
  drift: number;
}

interface SyncRowProps {
  drift: number;
  actions: AccountActions;
  onCompare: () => void;
}

interface AccountCardProps {
  account: Account;
  actions: AccountActions;
}

interface LimitMeterProps {
  limit: AccountLimit;
}

interface AccountFactsProps {
  account: Account;
}

interface RuleChipsProps {
  account: Account;
}

export type { Account, AccountActions, AccountCardProps, AccountFactsProps, AccountsFeed, AccountsSummaryProps, AccountsView, LimitMeterProps, RuleChipsProps, SyncRowProps };
