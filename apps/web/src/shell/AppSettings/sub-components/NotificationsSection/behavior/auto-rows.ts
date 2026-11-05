/* @layer renderer-app @kind logic */
import type { SettingsSectionRow } from '@drizztdourden08/tessera';
import type { NotifySettingsHook } from '../../../../../data/notify';
import { AUTO_PINGS } from '../NotificationsSection.constants';
import type { Report } from '../NotificationsSection.type';

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

export { autoRows };
