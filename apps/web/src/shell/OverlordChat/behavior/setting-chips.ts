/* @layer renderer-app @kind logic */
import type { OverlordSettings } from '../../../data/overlord';
import { SETTING_LABELS } from '../OverlordChat.constants';
import type { ChipSpec, SettingChoice } from '../chat-settings.type';
import { capitalize } from './capitalize';
import { labelOf } from './label-of';

const effortChoices = (efforts: readonly string[]): SettingChoice[] => efforts.map((id) => ({ id, label: capitalize(id) }));

const settingChips = (settings: OverlordSettings): ChipSpec[] => {
  const { models, efforts, permissionModes } = settings.choices;
  return [
    { key: 'model', label: SETTING_LABELS.model, current: labelOf(models, settings.model), selected: settings.model, choices: models },
    { key: 'effort', label: SETTING_LABELS.effort, current: capitalize(settings.effort), selected: settings.effort, choices: effortChoices(efforts) },
    {
      key: 'permissionMode',
      label: SETTING_LABELS.permissionMode,
      current: labelOf(permissionModes, settings.permissionMode),
      selected: settings.permissionMode,
      choices: permissionModes,
    },
  ];
};

export { settingChips };
