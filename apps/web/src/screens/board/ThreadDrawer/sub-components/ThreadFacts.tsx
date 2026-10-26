/* @layer renderer-app @kind component */
import { DateText, Flex, InlineEdit, Status, Tag, Text } from '@drizztdourden08/tessera';
import { activityTone, nonEmpty } from '../../shared';
import { validateField } from '../../tabs/kanban';
import { COORDINATOR_ROLE } from '../ThreadDrawer.constants';
import type { ThreadFactsProps } from '../ThreadDrawer.type';

const validateSubject = validateField('subject');

const ThreadFacts = (props: ThreadFactsProps) => {
  const { thread, counts, ops } = props;
  return (
    <Flex direction="column" gap="xs">
      <Flex gap="xs" align="center" wrap>
        <Tag>{thread.status}</Tag>
        <Status tone={activityTone(thread.activity)} dot>{nonEmpty(thread.activity, 'unknown')}</Status>
        {thread.role === COORDINATOR_ROLE && <Tag variant="urgency" color="info">coord</Tag>}
        {thread.archivedAt && <Tag>archived</Tag>}
        {counts.questions > 0 && <Tag variant="urgency" color="warning">{`${counts.questions} open`}</Tag>}
        {counts.toPickUp > 0 && <Tag variant="urgency" color="warning">{`${counts.toPickUp} to pick up`}</Tag>}
      </Flex>
      <Flex gap="md" align="center" wrap>
        <Flex gap="xs" align="center">
          <Text variant="caption" tone="muted">Subject</Text>
          <InlineEdit
            label="subject"
            placeholder="add subject"
            value={thread.subject ?? ''}
            validate={validateSubject}
            onCommit={(value) => ops.setField(thread, 'subject', value.trim())}
          />
        </Flex>
        <Text variant="caption" tone="muted">
          seen {thread.lastSeen ? <DateText value={thread.lastSeen} format="relative" /> : 'never'}
        </Text>
      </Flex>
      {thread.root && <Text variant="caption" tone="muted" mono>{thread.root}</Text>}
    </Flex>
  );
};

export { ThreadFacts };
