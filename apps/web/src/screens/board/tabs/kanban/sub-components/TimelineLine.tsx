/* @layer renderer-app @kind component */
import { Button, Flex, Tag, Text } from '@drizztdourden08/tessera';
import { BoardMarkdown } from '../../../../../shell';
import { formatAgo } from '../../../../../format';
import type { TimelineLineProps } from '../drawer-props.type';
import { EntryLink } from '../../../shared';
import { KIND_LABEL, TIMELINE_KINDS } from '../record';

const TimelineLine = (props: TimelineLineProps) => {
  const { row, thread, acts, now } = props;
  const todo = { id: row.id, item: null, thread };
  const linked = TIMELINE_KINDS.includes(row.kind);
  return (
    <Flex gap="sm" align="center" wrap role="listitem">
      <Text variant="caption" tone="muted">{formatAgo(row.at, now)}</Text>
      <Tag>{KIND_LABEL[row.kind] ?? row.kind}</Tag>
      {row.who !== thread && <Text variant="caption" tone="muted">{row.who}</Text>}
      {row.id && linked && <EntryLink id={row.id} />}
      {row.id && !linked && <Text variant="caption" tone="muted" mono>{row.id}</Text>}
      {row.on && <Text variant="caption" tone="muted">{`on ${row.on}`}</Text>}
      <BoardMarkdown size="sm" source={row.text} />
      {row.status && <Tag>{row.status}</Tag>}
      {row.doneable && (
        <Button size="xs" variant="primary" aria-label={`Mark ${row.id} done`} disabled={acts.isBusy(`post:done:${row.id}`)} onClick={() => void acts.done(todo)}>
          Done
        </Button>
      )}
    </Flex>
  );
};

export { TimelineLine };
