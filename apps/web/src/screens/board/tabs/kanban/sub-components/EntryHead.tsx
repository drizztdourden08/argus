/* @layer renderer-app @kind component */
import { Flex, Spacer, Tag, Text } from '@drizztdourden08/tessera';
import { askMode } from '../../../feed-model';
import { CardMenu } from '../../../OwnerFeed';
import { formatAgo } from '../../../../../format';
import type { EntryHeadProps } from '../drawer-props.type';
import { KIND_LABEL } from '../record';

const EntryHead = (props: EntryHeadProps) => {
  const { entry, feed, now } = props;
  const waiting = entry.kind === 'decision' && entry.author === 'human' && entry.status === 'pending';
  return (
    <Flex gap="xs" align="center" wrap>
      <Text variant="caption" tone="muted" mono>{entry.id}</Text>
      <Tag>{KIND_LABEL[entry.kind] ?? entry.kind}</Tag>
      {entry.kind === 'decision' && entry.replyTo && <Text variant="caption" tone="muted">{`· answers ${entry.replyTo}`}</Text>}
      {askMode(entry) === 'decide' && <Text variant="caption" tone="muted">· needs a decision</Text>}
      <Text variant="caption" tone="muted">{`· ${entry.author === 'human' ? 'you' : entry.author} · ${formatAgo(entry.createdAt, now)}`}</Text>
      {entry.status && <Tag>{entry.status}</Tag>}
      {waiting && <Text variant="caption" tone="muted">{`waiting for ${entry.thread} to pick it up`}</Text>}
      <Spacer />
      <CardMenu collection="entries" id={entry.id} actions={feed} />
    </Flex>
  );
};

export { EntryHead };
