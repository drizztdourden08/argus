/* @layer renderer-app @kind logic */
import { useCallback, useEffect, useState } from 'react';
import { apiRequest, errorMessage } from '../api';
import type { AccountsSettings, SettingsAccount } from '../types';
import { SETTINGS_PATH } from './accounts-settings.constants';
import type { AccountsSettingsApi } from './accounts-settings.type';

const useAccountsSettings = (): AccountsSettingsApi => {
  const [settings, setSettings] = useState<AccountsSettings | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let live = true;
    apiRequest<AccountsSettings>(SETTINGS_PATH).then(
      (read) => live && setSettings(read),
      (failure: unknown) => live && setProblem(errorMessage(failure)),
    );
    return () => {
      live = false;
    };
  }, []);

  const save = useCallback(async (patch: Partial<AccountsSettings>): Promise<boolean> => {
    setBusy(true);
    try {
      setSettings(await apiRequest<AccountsSettings>(SETTINGS_PATH, { method: 'POST', json: patch }));
      setProblem(null);
      return true;
    } catch (failure) {
      setProblem(errorMessage(failure));
      return false;
    } finally {
      setBusy(false);
    }
  }, []);

  const changeAccount = useCallback(
    async (key: string, change: Partial<SettingsAccount>): Promise<boolean> => {
      if (!settings) return false;
      const accounts = settings.accounts.map((item) => (item.key === key ? { ...item, ...change } : item));
      return save({ accounts });
    },
    [settings, save],
  );

  return { settings, problem, busy, save, changeAccount };
};

export { useAccountsSettings };
