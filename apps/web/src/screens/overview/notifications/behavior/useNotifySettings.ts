/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useEffect, useState } from 'react';
import { readSettings, writeSettings } from '../../../../data/notify';
import type { NotifyLevel, NotifySettings } from '../../../../data/types';
import { apiSentence } from '../model';
import type { SettingPart } from '../model';
import type { SettingsState } from '../notifications.type';

const useNotifySettings = (): SettingsState => {
  const [settings, setSettings] = useState<NotifySettings | null>(null);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let live = true;
    readSettings().then(
      (read) => live && setSettings(read),
      (failure: unknown) => live && setError(apiSentence(failure, (detail) => `The settings could not be read: ${detail}.`)),
    );
    return () => {
      live = false;
    };
  }, []);

  const toggle = useCallback((part: SettingPart, level: NotifyLevel) => {
    if (busy || !settings) return;
    setBusy(true);
    writeSettings({ [part]: { [level]: !settings[part]?.[level] } })
      .then(
        (saved) => setSettings(saved),
        (failure: unknown) => toast({ variant: 'danger', message: apiSentence(failure, (detail) => `Not saved: ${detail}.`) }),
      )
      .finally(() => setBusy(false));
  }, [busy, settings]);

  return { settings, error, busy, toggle };
};

export { useNotifySettings };
