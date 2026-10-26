/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { askMode } from '../../../feed-model';
import { formatAgo } from '../../../../../format';
import type { EntryHeadProps } from '../drawer-props.type';
import { KIND_LABEL } from '../record';

const EntryHead = (props: EntryHeadProps) => {
  const { entry, now } = props;
  const waiting = entry.kind === 'decision' && entry.author === 'human' && entry.status === 'pending';
  return (
    <Flex gap="xs" align="center" wrap>
      <Text variant="caption" tone="muted" mono>{entry.id}</Text>
      <Tag>{KIND_LABEL[entry.kind] ?? entry.kind}</Tag>
      {entry.kind === 'decision' && entry.replyTo && <Text variant="caption" tone="muted">{`answers ${entry.replyTo}`}</Text>}
      {askMode(entry) === 'decide' && <Text variant="caption" tone="muted">needs a decision</Text>}
      <Text variant="caption" tone="muted">{`${entry.author === 'human' ? 'you' : entry.author}, ${formatAgo(entry.createdAt, now)} ago`}</Text>
      {entry.status && <Tag>{entry.status}</Tag>}
      {waiting && <Text variant="caption" tone="muted">{`waiting for ${entry.thread} to pick it up`}</Text>}
    </Flex>
  );
};

export { EntryHead };
