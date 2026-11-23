/* @layer renderer-app @kind component */
import { Button, Callout, NumberInput, Stack, TextInput } from '@drizztdourden08/tessera';
import { Panel } from '../../Panel';
import { useSettingsForm } from '../behavior/useSettingsForm';
import { SettingsField } from './SettingsField';

const SettingsTab = () => {
  const form = useSettingsForm();
  const { settings, draft, action } = form;
  const failure = action.problem && <Callout tone="danger">{action.problem}</Callout>;
  if (!settings || !draft) return <Panel title="Settings">{failure}</Panel>;
  const field = { busy: action.busy, onReset: form.reset };
  return (
    <Panel title="Settings">
      <Stack gap="md">
        {failure}
        <SettingsField name="cwd" source={settings.cwd.source} {...field}>
          <TextInput value={draft.cwd} onChange={(event) => form.change({ cwd: event.target.value })} />
        </SettingsField>
        <SettingsField name="claudeExe" source={settings.claudeExe.source} {...field}>
          <TextInput value={draft.claudeExe} onChange={(event) => form.change({ claudeExe: event.target.value })} />
        </SettingsField>
        <SettingsField name="budgetUsd" source={settings.budgetUsd.source} {...field}>
          <NumberInput sizeToContent min={1} max={10000} value={draft.budgetUsd} onChange={(budgetUsd) => form.change({ budgetUsd })} />
        </SettingsField>
        <SettingsField name="restartMax" source={settings.restartMax.source} {...field}>
          <NumberInput sizeToContent min={0} max={20} value={draft.restartMax} onChange={(restartMax) => form.change({ restartMax })} />
        </SettingsField>
        <SettingsField name="approvalTimeoutMin" source={settings.approvalTimeoutMin.source} {...field}>
          <NumberInput
            sizeToContent
            min={1}
            max={1440}
            value={draft.approvalTimeoutMin}
            onChange={(approvalTimeoutMin) => form.change({ approvalTimeoutMin })}
          />
        </SettingsField>
        <Button variant="primary" disabled={action.busy} onClick={form.save}>Save</Button>
      </Stack>
    </Panel>
  );
};

export { SettingsTab };
