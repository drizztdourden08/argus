/* @layer renderer-app @kind types */
import type { AccountsSettings, SettingsAccount } from '../types';

interface AccountsSettingsApi {
  settings: AccountsSettings | null;
  problem: string | null;
  busy: boolean;
  save: (patch: Partial<AccountsSettings>) => Promise<boolean>;
  changeAccount: (key: string, change: Partial<SettingsAccount>) => Promise<boolean>;
}

export type { AccountsSettingsApi };
