/* @layer renderer-app @kind logic */
import type { SettingsSectionRow } from '@drizztdourden08/tessera';
import type { NotifySettingsHook } from '../../../../../data/notify';
import type { Report } from '../NotificationsSection.type';

const osRow = (s: NotifySettingsHook, report: Report): SettingsSectionRow => ({
  id: 'os',
  title: 'Notifications on',
  description: 'Off records them without any toast.',
  input: { kind: 'toggle', value: s.os, onChange: (os) => void s.saveAuto({ os }).then((r) => report('os', r)) },
});

export { osRow };
