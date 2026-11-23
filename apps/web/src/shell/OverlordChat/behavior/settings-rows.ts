/* @layer renderer-app @kind logic */
import type { SettingsInput, SettingsSectionRow } from '@drizztdourden08/tessera';
import type { LimitKey } from '../../../data/overlord';
import { SETTINGS_ROWS, SOURCE_CAPTIONS } from '../OverlordChat.constants';
import type { SettingsFormState } from './chat-settings.type';

const settingsRow = (form: SettingsFormState, key: LimitKey, input: SettingsInput): SettingsSectionRow => {
  const { title, note } = SETTINGS_ROWS[key];
  const source = form.settings ? form.settings[key].source : 'default';
  return {
    id: key,
    title,
    description: `${note} ${SOURCE_CAPTIONS[source]}.`,
    changed: source === 'set',
    onReset: form.action.busy ? undefined : () => form.reset(key),
    input,
  };
};

export { settingsRow };
