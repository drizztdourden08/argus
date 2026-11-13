/* @layer renderer-app @kind component */
import { Tag, Text } from '@drizztdourden08/tessera';
import { useBoardName } from '../../../../../data/boards';
import { askMode } from '../../../feed-model';
import { RecordLink } from '../../../../../shell';
import { formatAgo } from '../../../../../format';
import type { EntryHeadProps } from '../drawer-props.type';
import { KIND_LABEL } from '../record';

const EntryHead = (props: EntryHeadProps) => {
  const { entry, now } = props;
  const nameOf = useBoardName();
  const waiting = entry.kind === 'decision' && entry.author === 'human' && entry.status === 'pending';
  return (
    <>
      <Tag>{KIND_LABEL[entry.kind] ?? entry.kind}</Tag>
      {entry.kind === 'decision' && entry.replyTo && <Text variant="caption" tone="muted">{'· answers '}<RecordLink id={entry.replyTo} /></Text>}
      {askMode(entry) === 'decide' && <Text variant="caption" tone="muted">· needs a decision</Text>}
      <Text variant="caption" tone="muted">{`· ${entry.author === 'human' ? 'you' : nameOf(entry.author)} · ${formatAgo(entry.createdAt, now)}`}</Text>
      {entry.status && <Tag>{entry.status}</Tag>}
      {waiting && <Text variant="caption" tone="muted">{`waiting for ${entry.thread} to pick it up`}</Text>}
    </>
  );
};

export { EntryHead };
