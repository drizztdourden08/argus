/* @layer renderer-app @kind logic */
import { useState } from 'react';
import { readSettings, saveSettings, useAction } from '../../../data/overlord';
import type { OverlordSettings } from '../../../data/overlord';
import { useRemoteRead } from '../../../data/remote';
import type { ChatSettings, SettingKey } from '../chat-settings.type';

const useChatSettings = (): ChatSettings => {
  const read = useRemoteRead(readSettings);
  const [saved, setSaved] = useState<OverlordSettings | null>(null);
  const action = useAction(() => undefined);

  const pick = (key: SettingKey, id: string) => {
    void action.run(async () => setSaved(await saveSettings({ [key]: id })));
  };

  return { settings: saved ?? read.data, busy: action.busy, problem: action.problem, pick };
};

export { useChatSettings };
