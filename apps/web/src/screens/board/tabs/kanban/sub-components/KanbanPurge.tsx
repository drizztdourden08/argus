/* @layer renderer-app @kind component */
import { focusKey, readStored } from '../../../shared';
import { useKanbanData } from '../behavior/useKanbanData';
import type { KanbanPurgeProps } from '../trash-props.type';
import { PurgeDialog } from './PurgeDialog';

const KanbanPurge = (props: KanbanPurgeProps) => {
  const { boardKey, request, onClose } = props;
  const data = useKanbanData(boardKey);
  if (!request) return null;
  const focus = readStored(focusKey(boardKey)) ?? '';
  const focused = data.threads.some((thread) => thread.id === focus && thread.role !== 'coordinator') ? focus : '';
  return <PurgeDialog boardKey={boardKey} data={data} thread={request.thread ?? focused} scope={request.scope} onClose={onClose} />;
};

export { KanbanPurge };
