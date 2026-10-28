/* @layer renderer-app @kind component */
import { Button, Checkbox, Flex, TableCell, TableRow, Text } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../../format';
import type { TrashLineProps } from '../trash-props.type';
import { TRASH_KINDS } from '../trash';

const TrashLine = (props: TrashLineProps) => {
  const { row, now, selected, selection, order, bulk } = props;
  const how = row.archived ? 'archived ' : '';
  const shown = `${how}${formatAgo(row.at, now)} · ${row.by}`;
  const when = `${how}${formatAgo(row.at, now)} ago by ${row.by}${row.reason ? `: ${row.reason}` : ''}`;
  return (
    <TableRow>
      <TableCell>
        <Flex onClickCapture={(event) => selection.mark(event.shiftKey)}>
          <Checkbox checked={selected} onChange={() => selection.toggle(row.key, order)} ariaLabel={`Select ${row.id}`} size="sm" />
        </Flex>
      </TableCell>
      <TableCell>{row.id}</TableCell>
      <TableCell>{TRASH_KINDS[row.c]}</TableCell>
      <TableCell>{row.text}</TableCell>
      <TableCell>{row.thread}</TableCell>
      <TableCell><Text as="span" title={when}>{shown}</Text></TableCell>
      <TableCell>
        <Button size="xs" variant="ghost" aria-label={`Restore ${row.id}`} onClick={() => bulk.restore([row])}>Restore</Button>
      </TableCell>
    </TableRow>
  );
};

export { TrashLine };
