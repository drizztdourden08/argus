/* @layer renderer-app @kind types */
import type { AccountsSettings, SettingsAccount } from '../../../../../data/types';

interface AccountsDraft {
  draft: AccountsSettings | null;
  patch: (change: Partial<AccountsSettings>) => void;
  changeAccount: (index: number, change: Partial<SettingsAccount>) => void;
  addAccount: () => void;
}

export type { AccountsDraft };
