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
}

interface AccountsSummaryProps {
  accounts: readonly Account[];
  drift: number;
}

interface SyncRowProps {
  drift: number;
  actions: AccountActions;
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

export type { Account, AccountActions, AccountCardProps, AccountFactsProps, AccountsFeed, AccountsSummaryProps, LimitMeterProps, RuleChipsProps, SyncRowProps };
