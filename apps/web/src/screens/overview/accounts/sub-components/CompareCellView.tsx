/* @layer renderer-app @kind component */
import { Inline, Text } from '@drizztdourden08/tessera';
import { GHOST_STATUSES, INDENT } from '../compare.constants';
import type { CompareCellViewProps } from '../compare.type';
import { CellChip } from './CellChip';

const CompareCellView = (props: CompareCellViewProps) => {
  const { row, cell } = props;
  if (!cell) return null;
  const status = cell.status;
  const ghost = status !== undefined && GHOST_STATUSES.has(status);
  const plus = ghost ? '+ ' : '';
  if (row.kind === 'folder') {
    return (
      <Inline gap="sm" align="center" wrap>
        <Text variant="overline" italic={ghost}>{plus}{row.title}</Text>
        <CellChip status={status} />
      </Inline>
    );
  }
  const archivedNote = cell.archived === true && status !== 'archived_here_only';
  return (
    <Inline gap="sm" align="center" wrap>
      <Text variant={ghost ? 'subtitle' : 'body'} italic={ghost}>{INDENT.repeat(row.depth)}{plus}{row.title}</Text>
      <CellChip status={status} />
      {archivedNote && <Text variant="caption">archived</Text>}
    </Inline>
  );
};

export { CompareCellView };
