/* @layer renderer-app @kind component */
import { Button, Callout, SettingsSection, Stack } from '@drizztdourden08/tessera';
import type { SettingsInput } from '@drizztdourden08/tessera';
import type { LimitKey } from '../../../data/overlord';
import { Panel } from '../../Panel';
import { settingsRow } from '../behavior/settings-rows';
import { useSettingsForm } from '../behavior/useSettingsForm';
import { SETTINGS_SECTIONS } from '../OverlordChat.constants';
import { OverlordLive } from './OverlordLive';

const SettingsTab = () => {
  const form = useSettingsForm();
  const { draft, action } = form;
  const failure = action.problem && <Callout tone="danger">{action.problem}</Callout>;
  if (!form.settings || !draft) return <Panel title="Settings"><Stack gap="md"><OverlordLive />{failure}</Stack></Panel>;
  const row = (key: LimitKey, input: SettingsInput) => settingsRow(form, key, input);
  return (
    <Panel title="Settings">
      <Stack gap="md">
        <OverlordLive />
        {failure}
        <SettingsSection
          title={SETTINGS_SECTIONS.agent}
          rows={[
            row('cwd', { kind: 'text', value: draft.cwd, onChange: (cwd) => form.change({ cwd }) }),
            row('exePath', { kind: 'text', value: draft.exePath, onChange: (exePath) => form.change({ exePath }) }),
          ]}
        />
        <SettingsSection
          title={SETTINGS_SECTIONS.limits}
          rows={[
            row('budgetUsd', { kind: 'number', min: 1, max: 10000, unit: 'USD', value: draft.budgetUsd, onChange: (budgetUsd) => form.change({ budgetUsd }) }),
            row('restartMax', { kind: 'number', min: 0, max: 20, value: draft.restartMax, onChange: (restartMax) => form.change({ restartMax }) }),
            row('approvalTimeoutMin', { kind: 'number', min: 1, max: 1440, unit: 'min', value: draft.approvalTimeoutMin, onChange: (approvalTimeoutMin) => form.change({ approvalTimeoutMin }) }),
          ]}
        />
        <Button variant="primary" disabled={action.busy} onClick={form.save}>Save</Button>
      </Stack>
    </Panel>
  );
};

export { SettingsTab };
