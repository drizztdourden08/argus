/* @layer renderer-app @kind component */
import { Inline, Text } from '@drizztdourden08/tessera';
import { StatusChip } from '../../sub-components/StatusChip';
import type { CellNotesProps } from '../review-props.type';

const CellNotes = (props: CellNotesProps) => {
  const { archived, note, pending } = props;
  return (
    <Inline gap="xs" align="center" wrap={false} className="review-tree-notes">
      {archived && <Text variant="caption" tone="muted" className="review-tree-note">{archived}</Text>}
      {note && <StatusChip look={{ label: note, tone: 'info' }} />}
      {pending && <StatusChip look={{ label: 'edit pending', tone: 'warning' }} />}
    </Inline>
  );
};

export { CellNotes };
