/* @layer renderer-app @kind component */
import { Button, Callout, EmptyState, Flex, Stack } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { Panel } from '../../Panel';
import { useRules } from '../behavior/useRules';
import { NO_RULES } from '../OverlordChat.constants';
import type { Editing } from '../behavior/chat-panels.type';
import { RuleForm } from './RuleForm';
import { RuleRow } from './RuleRow';

const RulesTab = () => {
  const rules = useRules();
  const [editing, setEditing] = useState<Editing>(null);
  const close = () => {
    setEditing(null);
    rules.reload();
  };
  if (editing) return <Panel title={editing === 'new' ? 'New rule' : 'Edit rule'}><RuleForm rule={editing === 'new' ? null : editing} onClose={close} /></Panel>;
  const add = <Button size="sm" variant="primary" onClick={() => setEditing('new')}>Add</Button>;
  return (
    <Panel title="Rules" tools={add}>
      <Stack gap="sm">
        {rules.problem && <Callout tone="danger">{rules.problem}</Callout>}
        {rules.action.problem && <Callout tone="danger">{rules.action.problem}</Callout>}
        {rules.rules.length === 0 && <EmptyState message={NO_RULES} size="sm" />}
        <Flex direction="column" gap="sm">
          {rules.rules.map((rule) => (
            <RuleRow
              key={rule.id}
              rule={rule}
              busy={rules.action.busy}
              onToggle={(enabled) => void rules.toggle(rule, enabled)}
              onEdit={() => setEditing(rule)}
              onRun={() => void rules.run(rule)}
              onDelete={() => void rules.remove(rule)}
            />
          ))}
        </Flex>
      </Stack>
    </Panel>
  );
};

export { RulesTab };
