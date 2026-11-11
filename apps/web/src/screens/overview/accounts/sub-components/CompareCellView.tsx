/* @layer renderer-app @kind component */
import { Inline, Text } from '@drizztdourden08/tessera';
import { GHOST_STATUSES, INDENT, QUIET_STATUSES, STATUS_LOOK } from '../compare.constants';
import type { CompareCellViewProps } from '../compare.type';
import { StatusChip } from './StatusChip';

const CompareCellView = (props: CompareCellViewProps) => {
  const { row, cell } = props;
  if (!cell) return null;
  const status = cell.status;
  const ghost = status !== undefined && GHOST_STATUSES.has(status);
  if (row.kind === 'folder') {
    return (
      <Inline gap="sm" align="center" wrap>
        <Text variant="overline" italic={ghost}>{ghost ? '+ ' : ''}{row.title}</Text>
        {status && !QUIET_STATUSES.has(status) && <StatusChip look={STATUS_LOOK[status]} />}
      </Inline>
    );
  }
  const archivedNote = cell.archived === true && status !== 'archived_here_only';
  return (
    <Inline gap="sm" align="center" wrap>
      <Text variant={ghost ? 'subtitle' : 'body'} italic={ghost}>{INDENT.repeat(row.depth)}{ghost ? '+ ' : ''}{row.title}</Text>
      {status && !QUIET_STATUSES.has(status) && <StatusChip look={STATUS_LOOK[status]} />}
      {archivedNote && <Text variant="caption">archived</Text>}
    </Inline>
  );
};

export { CompareCellView };
