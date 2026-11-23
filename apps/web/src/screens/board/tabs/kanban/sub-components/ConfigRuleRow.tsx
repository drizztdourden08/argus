/* @layer renderer-app @kind component */
import { Disclosure, Field, Flex, Icon, IconButton, Select, Text, TextInput } from '@drizztdourden08/tessera';
import { RULE_KINDS, configRuleOfKind } from '../model';
import type { ConfigRuleRowProps } from '../config-editor.type';
import { ConfigRuleParams } from './ConfigRuleParams';
import { ConfigRuleReaction } from './ConfigRuleReaction';

const KIND_OPTIONS = RULE_KINDS.map((entry) => ({ value: entry.kind, label: entry.kind }));

const ConfigRuleRow = (props: ConfigRuleRowProps) => {
  const { rule, onChange, onRemove } = props;
  return (
    <Disclosure
      size="sm"
      summary={<Flex gap="sm" align="center"><Text variant="label">{rule.id}</Text><Text variant="caption" tone="muted">{rule.kind}</Text></Flex>}
      end={<IconButton label={`Remove rule ${rule.id}`} variant="ghost" size="sm" onClick={onRemove}><Icon name="trash" /></IconButton>}
    >
      <Flex direction="column" gap="md">
        <Flex gap="md">
          <Field label="Rule id" width="sm">
            <TextInput value={rule.id} onChange={(event) => onChange({ ...rule, id: event.target.value })} />
          </Field>
          <Field label="Kind" width="sm">
            <Select options={KIND_OPTIONS} value={rule.kind} onChange={(kind) => onChange(configRuleOfKind(rule, kind))} />
          </Field>
        </Flex>
        <ConfigRuleParams rule={rule} onChange={onChange} />
        <ConfigRuleReaction rule={rule} onChange={onChange} />
      </Flex>
    </Disclosure>
  );
};

export { ConfigRuleRow };
