/* @layer renderer-app @kind component */
import { SettingsSection } from '@drizztdourden08/tessera';
import type { SettingsSectionRow } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useNotifySettings } from '../../../../data/notify';
import { autoRows } from './behavior/auto-rows';
import { levelRows } from './behavior/level-rows';
import { osRow } from './behavior/os-row';
import { useTestNotification } from './behavior/useTestNotification';

const withProblem = (rows: SettingsSectionRow[], refused: Record<string, string>): SettingsSectionRow[] =>
  rows.map((row) => (refused[row.id] ? { ...row, problem: refused[row.id] } : row));

const NotificationsSection = () => {
  const settings = useNotifySettings();
  const [refused, setRefused] = useState<Record<string, string>>({});
  const test = useTestNotification(settings.sendTest);
  const report = (id: string, refusal: string | null) => setRefused((was) => ({ ...was, [id]: refusal ?? '' }));
  const testRow: SettingsSectionRow = {
    id: 'test',
    title: 'Send a test notification',
    description: test.line,
    actions: [{ id: 'send', label: 'Send', loading: test.busy, onSelect: test.send }],
  };
  const top = withProblem([osRow(settings, report), ...levelRows(settings, report)], refused);
  return (
    <>
      <SettingsSection id="notifications-os" title="OS notifications" rows={top} />
      <SettingsSection id="notifications-auto" title="Automatic pings" rows={withProblem(autoRows(settings, report), refused)} />
      <SettingsSection id="notifications-test" rows={[testRow]} />
    </>
  );
};

export { NotificationsSection };
