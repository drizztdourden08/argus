/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { StatusChip } from '../../sub-components/StatusChip';
import type { CellNotesProps } from '../review-props.type';

const CellNotes = (props: CellNotesProps) => {
  const { archived, note, pending } = props;
  return (
    <>
      {archived && <Text variant="caption" tone="muted">{archived}</Text>}
      {note && <StatusChip look={{ label: note, tone: 'info' }} />}
      {pending && <StatusChip look={{ label: 'edit pending', tone: 'warning' }} />}
    </>
  );
};

export { CellNotes };
