/* @layer renderer-app @kind component */
import { Flex, Select, Text } from '@drizztdourden08/tessera';
import type { DrawerFactsProps } from '../kanban-props.type';
import { STATUS_OPTIONS, ago, threadChoices } from '../model';
import { InlineEdit } from './InlineEdit';

const DrawerFacts = (props: DrawerFactsProps) => {
  const { item, threads, now, ops } = props;
  const subject = item.subject ?? '';
  return (
    <Flex direction="column" gap="sm">
      <Flex gap="md" align="center" wrap>
        <Select
          aria-label="Status"
          size="sm"
          options={STATUS_OPTIONS}
          value={item.status}
          onChange={(status) => status !== item.status && void ops.move(item, { status }, `Moved ${item.id}.`)}
        />
        <Select
          aria-label="Thread"
          size="sm"
          options={threadChoices(threads, item.thread)}
          value={item.thread}
          onChange={(thread) => thread !== item.thread && void ops.move(item, { thread }, `Moved ${item.id}.`)}
        />
        <Flex gap="xs" align="center">
          <Text variant="caption" tone="muted">Subject</Text>
          <InlineEdit
            label="subject"
            field="subject"
            value={subject}
            display={<Text>{subject || 'none'}</Text>}
            onSave={(value) => void ops.setField(item, 'subject', value)}
          />
        </Flex>
      </Flex>
      <Text variant="caption" tone="muted">{`rev ${item.rev ?? 1}, updated by ${item.updatedBy ?? item.createdBy ?? 'unknown'} ${ago(item.updatedAt, now)} ago`}</Text>
    </Flex>
  );
};

export { DrawerFacts };
