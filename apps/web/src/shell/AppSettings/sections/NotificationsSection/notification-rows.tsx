/* @layer renderer-app @kind logic */
import type { SettingsSectionRow } from '@drizztdourden08/tessera';
import type { NotifySettingsHook } from '../../../../data/notify';
import { LevelControl } from './LevelControl';
import { AUTO_PINGS, LEVELS } from './notifications.constants';

type Report = (id: string, refusal: string | null) => void;

const osRow = (s: NotifySettingsHook, report: Report): SettingsSectionRow => ({
  id: 'os',
  title: 'Notifications on',
  description: 'Off records them without any toast.',
  input: { kind: 'toggle', value: s.os, onChange: (os) => void s.saveAuto({ os }).then((r) => report('os', r)) },
});

const levelRows = (s: NotifySettingsHook, report: Report): SettingsSectionRow[] =>
  LEVELS.map(({ level, title }) => ({
    id: level,
    title,
    noDescription: true,
    input: {
      kind: 'custom',
      control: (
        <LevelControl
          title={title}
          toast={s.toast[level] === true}
          sound={s.sound[level] === true}
          onToast={(toast) => void s.saveLevel(level, { toast }).then((r) => report(level, r))}
          onSound={(sound) => void s.saveLevel(level, { sound }).then((r) => report(level, r))}
        />
      ),
    },
  }));

const autoRows = (s: NotifySettingsHook, report: Report): SettingsSectionRow[] =>
  AUTO_PINGS.map(({ key, title }) => ({
    id: `auto-${key}`,
    title,
    noDescription: true,
    input: {
      kind: 'toggle',
      value: s.auto[key],
      onChange: (on) => void s.saveAuto({ auto: { [key]: on } }).then((r) => report(`auto-${key}`, r)),
    },
  }));

export { autoRows, levelRows, osRow };
export type { Report };
