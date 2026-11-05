/* @layer renderer-app @kind logic */
import { useEffect, useSyncExternalStore } from 'react';
import type { NotifySettingsHook } from '../notify.type';
import { sendTest } from '../send-test';
import { getState, load, saveAuto, saveLevel, subscribe } from '../settings-store';

const useNotifySettings = (): NotifySettingsHook => {
  const state = useSyncExternalStore(subscribe, getState);
  useEffect(() => {
    void load();
  }, []);
  return { ...state, saveLevel, saveAuto, sendTest };
};

export { useNotifySettings };
