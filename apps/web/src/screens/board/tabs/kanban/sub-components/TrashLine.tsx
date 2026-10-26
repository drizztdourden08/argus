/* @layer renderer-app @kind component */
import { Button, Checkbox, Flex, TableCell, TableRow } from '@drizztdourden08/tessera';
import type { TrashLineProps } from '../trash-props.type';
import { ago } from '../model';
import { TRASH_KINDS } from '../trash';

const TrashLine = (props: TrashLineProps) => {
  const { row, now, selected, selection, order, bulk } = props;
  const when = `${row.archived ? 'archived ' : ''}${ago(row.at, now)} ago`;
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
      <TableCell>{`${when} by ${row.by}${row.reason ? `: ${row.reason}` : ''}`}</TableCell>
      <TableCell>
        <Button size="xs" variant="ghost" aria-label={`Restore ${row.id}`} onClick={() => bulk.restore([row])}>Restore</Button>
      </TableCell>
    </TableRow>
  );
};

export { TrashLine };
