/* @layer renderer-app @kind component */
import { Button, Field, Flex, Select, TextInput } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { RuleCard } from '../../../../../shell/RuleCard';
import { configReactionText, configRuleFacts, configRuleOfKind } from '../model';
import type { ConfigRuleCardProps } from '../config-editor.type';
import { KIND_OPTIONS } from './ConfigRuleCard.constants';
import { ConfigRuleParams } from './ConfigRuleParams';
import { ConfigRuleReaction } from './ConfigRuleReaction';

const ConfigRuleCard = (props: ConfigRuleCardProps) => {
  const { rule, detachLabel, onChange, onRemove, onDetach } = props;
  const [editing, setEditing] = useState(false);
  return (
    <RuleCard
      title={rule.id}
      kind={rule.kind}
      facts={configRuleFacts(rule)}
      reaction={configReactionText(rule)}
      editLabel={`Edit rule ${rule.id}`}
      removeLabel={`Remove rule ${rule.id}`}
      onEdit={() => setEditing((was) => !was)}
      onRemove={onRemove}
      actions={onDetach && <Button size="xs" onClick={onDetach}>{detachLabel}</Button>}
      editing={editing && (
        <Flex direction="column" gap="md">
          <Field label="Rule id">
            <TextInput value={rule.id} onChange={(event) => onChange({ ...rule, id: event.target.value })} />
          </Field>
          <Field label="Kind">
            <Select options={KIND_OPTIONS} value={rule.kind} onChange={(kind) => onChange(configRuleOfKind(rule, kind))} />
          </Field>
          <ConfigRuleParams rule={rule} onChange={onChange} />
          <ConfigRuleReaction rule={rule} onChange={onChange} />
        </Flex>
      )}
    />
  );
};

export { ConfigRuleCard };
