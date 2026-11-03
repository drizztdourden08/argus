/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { BoardMarkdown, RecordLink } from '../../../../../shell';
import { formatAgo } from '../../../../../format';
import type { EntryReplyProps } from '../drawer-props.type';
import { KIND_LABEL } from '../record';

const EntryReply = (props: EntryReplyProps) => {
  const { reply, now } = props;
  return (
    <Flex direction="column" gap="2xs" role="group" aria-label={`Reply ${reply.id}`}>
      <Flex gap="xs" align="center" wrap>
        <RecordLink id={reply.id} />
        <Tag>{KIND_LABEL[reply.kind] ?? reply.kind}</Tag>
        <Text variant="caption" tone="muted">{`${reply.author === 'human' ? 'you' : reply.author} · ${formatAgo(reply.createdAt, now)}`}</Text>
        {reply.status && <Tag>{reply.status}</Tag>}
      </Flex>
      <BoardMarkdown source={reply.text} />
    </Flex>
  );
};

export { EntryReply };
