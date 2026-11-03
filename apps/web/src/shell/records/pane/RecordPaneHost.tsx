/* @layer renderer-app @kind component */
import { useLegacyHash } from '../behavior/useLegacyHash';
import { useRecordPane } from '../behavior/useRecordPane';
import type { RecordPaneHostProps } from './pane.type';
import { RecordPane } from './RecordPane';

const RecordPaneHost = (props: RecordPaneHostProps) => {
  const { ref } = useRecordPane();
  useLegacyHash();
  const board = ref?.board ?? props.boardKey ?? null;
  if (!ref || !board) return null;
  return <RecordPane boardKey={board} target={ref} />;
};

export { RecordPaneHost };
