/* @layer renderer-app @kind component */
import { Flex, InlineEdit, Text } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../format';
import { validateField } from '../../tabs/kanban/model';
import type { ThreadFactsProps } from '../ThreadDrawer.type';

const validateSubject = validateField('subject');

const ThreadFacts = (props: ThreadFactsProps) => {
  const { thread, now, ops } = props;
  return (
    <Flex direction="column" gap="xs">
      <Flex gap="md" align="center" wrap>
        <Flex gap="xs" align="center">
          <Text as="span" variant="caption">Subject:</Text>
          <InlineEdit
            label="subject"
            look="link"
            placeholder="add subject"
            value={thread.subject ?? ''}
            validate={validateSubject}
            onCommit={(value) => ops.setField(thread, 'subject', value.trim())}
          />
        </Flex>
        <Text as="span" variant="caption">{`seen ${formatAgo(thread.lastSeen, now)} ago`}</Text>
      </Flex>
      {thread.root && <Text variant="caption" tone="muted" mono>{thread.root}</Text>}
    </Flex>
  );
};

export { ThreadFacts };
