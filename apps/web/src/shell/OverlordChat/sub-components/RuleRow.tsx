/* @layer renderer-app @kind component */
import { Button, Flex, Stack, Tag, Text, Toggle } from '@drizztdourden08/tessera';
import { nextRunText } from '../behavior/next-run-text';
import type { RuleRowProps } from '../behavior/chat-panels.type';

const RuleRow = (props: RuleRowProps) => {
  const { rule, busy, onToggle, onEdit, onRun, onDelete } = props;
  return (
    <Stack gap="sm" className="overlord-chat__row">
      <Flex justify="between" align="center" gap="md" wrap>
        <Flex gap="sm" align="center" wrap>
          <Text variant="label">{rule.title}</Text>
          <Tag>{rule.createdBy}</Tag>
        </Flex>
        <Toggle label="Enabled" checked={rule.enabled} disabled={busy} onChange={onToggle} />
      </Flex>
      <Text variant="caption" tone="muted">Every {rule.everyMin} min. Next run {nextRunText(rule, Date.now())}.</Text>
      <Flex gap="sm" wrap>
        <Button size="xs" onClick={onEdit}>Edit</Button>
        <Button size="xs" disabled={busy} onClick={onRun}>Run now</Button>
        <Button size="xs" variant="danger" disabled={busy} onClick={onDelete}>Delete</Button>
      </Flex>
    </Stack>
  );
};

export { RuleRow };
