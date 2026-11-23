/* @layer renderer-app @kind logic */
import { apiRequest, errorMessage } from '../api';
import type { AccountsSettings, SettingsAccount } from '../types';
import { SETTINGS_PATH } from './accounts-settings.constants';
import type { AccountChange, AccountsSettingsState } from './accounts-settings.type';

const EMPTY: AccountsSettingsState = { settings: null, problem: null, busy: false };

let state = EMPTY;
let writes = 0;
let pending = 0;
const listeners = new Set<() => void>();

const publish = (change: Partial<AccountsSettingsState>): void => {
  state = { ...state, ...change };
  listeners.forEach((listener) => listener());
};

const getState = (): AccountsSettingsState => state;

const subscribe = (listener: () => void): (() => boolean) => {
  listeners.add(listener);
  return () => listeners.delete(listener);
};

const load = async (): Promise<void> => {
  const before = writes;
  try {
    const read = await apiRequest<AccountsSettings>(SETTINGS_PATH);
    if (writes === before) publish({ settings: read, problem: null });
  } catch (failure) {
    if (writes === before) publish({ problem: errorMessage(failure) });
  }
};

const save = async (patch: Partial<AccountsSettings>): Promise<boolean> => {
  writes += 1;
  pending += 1;
  publish({ busy: true });
  try {
    const saved = await apiRequest<AccountsSettings>(SETTINGS_PATH, { method: 'POST', json: patch });
    pending -= 1;
    publish({ settings: saved, problem: null, busy: pending > 0 });
    return true;
  } catch (failure) {
    pending -= 1;
    publish({ problem: errorMessage(failure), busy: pending > 0 });
    return false;
  }
};

const merged = (item: SettingsAccount, change: AccountChange): SettingsAccount => ({
  ...item,
  ...change,
  rules: { ...item.rules, ...change.rules },
});

const changeAccount = (key: string, change: AccountChange): Promise<boolean> => {
  const { settings } = state;
  if (!settings) return Promise.resolve(false);
  return save({ accounts: settings.accounts.map((item) => (item.key === key ? merged(item, change) : item)) });
};

const resetStore = (): void => {
  state = EMPTY;
  writes = 0;
  pending = 0;
};

const accountsSettingsStore = { changeAccount, getState, load, resetStore, save, subscribe };

export { accountsSettingsStore };
