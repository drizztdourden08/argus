/* @layer renderer-app @kind types */
import type { AccountsSettings, SettingsAccount, SettingsRules } from '../types';

type AccountChange = Partial<Omit<SettingsAccount, 'rules'>> & { rules?: Partial<SettingsRules> };

interface AccountsSettingsState {
  settings: AccountsSettings | null;
  problem: string | null;
  busy: boolean;
}

interface AccountsSettingsApi extends AccountsSettingsState {
  save: (patch: Partial<AccountsSettings>) => Promise<boolean>;
  changeAccount: (key: string, change: AccountChange) => Promise<boolean>;
}

export type { AccountChange, AccountsSettingsApi, AccountsSettingsState };
