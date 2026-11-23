/* @layer renderer-app @kind component */
import { Field, TextInput } from '@drizztdourden08/tessera';
import type { KanbanRuleConfig } from '../../../../../data/types';
import type { ConfigRuleParamProps } from '../config-editor.type';
import { ConfigHoursInput } from './ConfigHoursInput';
import { ConfigListInput } from './ConfigListInput';

const withParam = (rule: KanbanRuleConfig, key: string, value: unknown): KanbanRuleConfig => ({ ...rule, params: { ...rule.params, [key]: value } });

const listOf = (value: unknown): string[] => (Array.isArray(value) ? value.map(String) : []);

const ConfigRuleParam = (props: ConfigRuleParamProps) => {
  const { rule, field, onChange } = props;
  const value = rule.params?.[field.key];
  if (field.type === 'hours') return <ConfigHoursInput label={field.label} ms={typeof value === 'number' ? value : 0} onChange={(ms) => onChange(withParam(rule, field.key, ms))} />;
  if (field.type === 'list') return <ConfigListInput label={field.label} value={listOf(value)} onCommit={(list) => onChange(withParam(rule, field.key, list))} />;
  return (
    <Field label={field.label}>
      <TextInput value={typeof value === 'string' ? value : ''} onChange={(event) => onChange(withParam(rule, field.key, event.target.value))} />
    </Field>
  );
};

export { ConfigRuleParam };
