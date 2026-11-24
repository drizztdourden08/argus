/* @layer renderer-app @kind component */
import { Field, Flex, Select, Text } from '@drizztdourden08/tessera';
import { configToggleList } from '../model';
import type { ConfigColumnRulesProps } from '../config-editor.type';
import { ConfigRuleCard } from './ConfigRuleCard';

const ConfigColumnRules = (props: ConfigColumnRulesProps) => {
  const { column, rules, onChange, onRuleChange, onRuleRemove } = props;
  const attached = column.rules ?? [];
  const free = rules.filter((rule) => !attached.includes(rule.id));
  const name = column.name || column.id;
  return (
    <Flex direction="column" gap="sm">
      <Text variant="label">Rules</Text>
      {rules.map((rule, index) => attached.includes(rule.id) && (
        <ConfigRuleCard
          key={index}
          rule={rule}
          detachLabel={`Detach ${rule.id} from ${name}`}
          onChange={(next) => onRuleChange(index, next)}
          onRemove={() => onRuleRemove(index)}
          onDetach={() => onChange({ ...column, rules: configToggleList(attached, rule.id, false) })}
        />
      ))}
      {attached.length === 0 && <Text variant="caption" tone="muted">No rules on this column.</Text>}
      {free.length > 0 && (
        <Field label="Attach a rule">
          <Select
            options={free.map((rule) => ({ value: rule.id, label: rule.id }))}
            value=""
            placeholder="Pick a rule"
            onChange={(id) => onChange({ ...column, rules: configToggleList(attached, id, true) })}
          />
        </Field>
      )}
    </Flex>
  );
};

export { ConfigColumnRules };
