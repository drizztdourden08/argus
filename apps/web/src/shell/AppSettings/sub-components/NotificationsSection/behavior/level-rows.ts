/* @layer renderer-app @kind logic */
import type { SettingsSectionRow } from '@drizztdourden08/tessera';
import { createElement } from 'react';
import type { NotifySettingsHook } from '../../../../../data/notify';
import { LEVELS } from '../NotificationsSection.constants';
import type { Report } from '../NotificationsSection.type';
import { LevelControl } from '../sub-components/LevelControl';

const levelRows = (s: NotifySettingsHook, report: Report): SettingsSectionRow[] =>
  LEVELS.map(({ level, title }) => ({
    id: level,
    title,
    noDescription: true,
    input: {
      kind: 'custom',
      control: createElement(LevelControl, {
        title,
        toast: s.toast[level] === true,
        sound: s.sound[level] === true,
        onToast: (toast) => void s.saveLevel(level, { toast }).then((r) => report(level, r)),
        onSound: (sound) => void s.saveLevel(level, { sound }).then((r) => report(level, r)),
      }),
    },
  }));

export { levelRows };
