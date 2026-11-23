/* @layer renderer-app @kind types */
import type { WithId } from '../../../data/react';
import type { AccountDoc, AccountsDrift } from '../../../data/types';

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

interface PostAction {
  busy: boolean;
  post: (path: string, json: object | undefined, done: string) => void;
}

type AccountsView = 'overview' | 'review';

interface RowState {
  receive: boolean;
  full: boolean;
  until: string;
  blocked: boolean;
}

interface PercentFieldProps {
  value: number;
  label: string;
  disabled: boolean;
  onCommit: (value: number) => void;
}

export type { Account, AccountActions, AccountsFeed, AccountsView, PercentFieldProps, PostAction, RowState };
