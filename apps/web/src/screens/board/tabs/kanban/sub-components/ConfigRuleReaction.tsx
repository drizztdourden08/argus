/* @layer renderer-app @kind component */
import { Field, Flex, Select, TextInput, Toggle } from '@drizztdourden08/tessera';
import { ALERT_TARGET_HINT, NEW_ALERT, WRITE_MODES, configKind } from '../model';
import type { ConfigRuleFieldProps } from '../config-editor.type';
import { ConfigHoursInput } from './ConfigHoursInput';

const MODE_OPTIONS = WRITE_MODES.map((mode) => ({ value: mode, label: mode }));

const ConfigRuleReaction = (props: ConfigRuleFieldProps) => {
  const { rule, onChange } = props;
  if (!configKind(rule.kind).sweep) {
    return (
      <Field label="Mode" width="sm">
        <Select options={MODE_OPTIONS} value={rule.mode ?? 'warn'} onChange={(mode) => onChange({ ...rule, mode: mode as 'warn' | 'refuse' })} />
      </Field>
    );
  }
  const alert = rule.alert ?? NEW_ALERT;
  return (
    <Flex gap="md" align="end" wrap>
      <Field label="Alert to" hint={ALERT_TARGET_HINT}>
        <TextInput value={alert.to} onChange={(event) => onChange({ ...rule, alert: { ...alert, to: event.target.value } })} />
      </Field>
      <ConfigHoursInput label="Repeat every (hours)" ms={alert.repeatMs} onChange={(repeatMs) => onChange({ ...rule, alert: { ...alert, repeatMs } })} />
      <Toggle label="Wake the thread" checked={alert.wake} onChange={(wake) => onChange({ ...rule, alert: { ...alert, wake } })} />
    </Flex>
  );
};

export { ConfigRuleReaction };
