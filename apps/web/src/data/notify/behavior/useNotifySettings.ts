/* @layer renderer-app @kind logic */
import { useEffect, useSyncExternalStore } from 'react';
import type { NotifySettingsHook } from '../notify.type';
import { sendTest } from '../send-test';
import { settingsStore } from '../settings-store';

const { getState, load, saveAuto, saveLevel, subscribe } = settingsStore;

const useNotifySettings = (): NotifySettingsHook => {
  const state = useSyncExternalStore(subscribe, getState);
  useEffect(() => {
    void load();
  }, []);
  return { ...state, saveLevel, saveAuto, sendTest };
};

export { useNotifySettings };
