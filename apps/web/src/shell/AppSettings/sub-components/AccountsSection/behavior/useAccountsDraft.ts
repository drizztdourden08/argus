/* @layer renderer-app @kind logic */
import { useCallback, useEffect, useState } from 'react';
import type { AccountsSettings, SettingsAccount } from '../../../../../data/types';
import { BLANK_ACCOUNT } from '../AccountsSection.constants';
import type { AccountsDraft } from './accounts-draft.type';

const useAccountsDraft = (saved: AccountsSettings | null): AccountsDraft => {
  const [draft, setDraft] = useState<AccountsSettings | null>(saved);
  useEffect(() => setDraft(saved), [saved]);
  const patch = useCallback((change: Partial<AccountsSettings>) => setDraft((was) => (was ? { ...was, ...change } : was)), []);
  const changeAccount = useCallback(
    (index: number, change: Partial<SettingsAccount>) =>
      setDraft((was) => (was ? { ...was, accounts: was.accounts.map((item, at) => (at === index ? { ...item, ...change } : item)) } : was)),
    [],
  );
  const addAccount = useCallback(
    () => setDraft((was) => (was ? { ...was, accounts: [...was.accounts, { ...BLANK_ACCOUNT, key: '', label: '', cliDir: '' }] } : was)),
    [],
  );
  return { draft, patch, changeAccount, addAccount };
};

export { useAccountsDraft };
