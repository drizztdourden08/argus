/* @layer renderer-app @kind types */
import type { AccountDoc, AccountsSettings, SettingsAccount } from '../../../../data/types';

interface AccountRowProps {
  item: SettingsAccount;
  doc: AccountDoc | undefined;
  isNew: boolean;
  onChange: (change: Partial<SettingsAccount>) => void;
}

interface DraftProps {
  draft: AccountsSettings;
  onChange: (change: Partial<AccountsSettings>) => void;
}

interface NumberFieldProps {
  label: string;
  value: number;
  step?: number;
  onChange: (value: number) => void;
}

export type { AccountRowProps, DraftProps, NumberFieldProps };
