/* @layer renderer-app @kind logic */
import { useState } from 'react';
import { readSettings, saveSettings, useAction } from '../../../data/overlord';
import type { OverlordSettings, OverlordSettingsPatch, SettingsDraft } from '../../../data/overlord';
import { useRemoteRead } from '../../../data/remote';
import type { SettingsFormState } from '../OverlordChat.type';
import { draftOf, patchOf } from './settings-patch';

const useSettingsForm = (): SettingsFormState => {
  const read = useRemoteRead(readSettings);
  const [saved, setSaved] = useState<OverlordSettings | null>(null);
  const [edited, setEdited] = useState<SettingsDraft | null>(null);
  const action = useAction(() => undefined);
  const settings = saved ?? read.data;
  const draft = edited ?? (settings ? draftOf(settings) : null);

  const send = (patch: OverlordSettingsPatch) => {
    void action.run(async () => {
      setSaved(await saveSettings(patch));
      setEdited(null);
    });
  };

  return {
    settings,
    draft,
    action,
    change: (patch) => draft && setEdited({ ...draft, ...patch }),
    save: () => settings && draft && send(patchOf(settings, draft)),
    reset: (key) => send({ [key]: '' }),
  };
};

export { useSettingsForm };
