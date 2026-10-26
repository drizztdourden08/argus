/* @layer renderer-app @kind component */
import { Button, Flex, Markdown, Tag, Text } from '@drizztdourden08/tessera';
import type { TimelineLineProps } from '../drawer-props.type';
import { ago } from '../model';
import { KIND_LABEL } from '../record';

const TimelineLine = (props: TimelineLineProps) => {
  const { row, thread, acts, now } = props;
  const todo = { id: row.id, item: null, thread };
  return (
    <Flex gap="sm" align="center" wrap role="listitem">
      <Text variant="caption" tone="muted">{`${ago(row.at, now)} ago`}</Text>
      <Tag>{KIND_LABEL[row.kind] ?? row.kind}</Tag>
      {row.who !== thread && <Text variant="caption" tone="muted">{row.who}</Text>}
      {row.id && <Text variant="caption" tone="muted" mono>{row.id}</Text>}
      {row.on && <Text variant="caption" tone="muted">{`on ${row.on}`}</Text>}
      <Markdown size="sm" source={row.text} />
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
