/* @layer renderer-app @kind types */
import type { AccountsView } from '../accounts.type';

interface AccountViewState {
  view: AccountsView;
  show: (next: AccountsView) => void;
}

export type { AccountViewState };
