/* @layer renderer-app @kind logic */
import { useEffect, useSyncExternalStore } from 'react';
import { accountsSettingsStore } from './accounts-settings-store';
import type { AccountsSettingsApi } from './accounts-settings.type';

const { changeAccount, getState, load, save, subscribe } = accountsSettingsStore;

const useAccountsSettings = (): AccountsSettingsApi => {
  const state = useSyncExternalStore(subscribe, getState);
  useEffect(() => {
    void load();
  }, []);
  return { ...state, save, changeAccount };
};

export { useAccountsSettings };
