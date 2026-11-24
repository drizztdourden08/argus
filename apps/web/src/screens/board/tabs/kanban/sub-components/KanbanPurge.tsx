/* @layer renderer-app @kind component */
import { focusKey, readStored } from '../../../shared';
import { useKanbanData } from '../behavior/useKanbanData';
import type { KanbanPurgeProps } from '../trash-props.type';
import { PurgeDialog } from './PurgeDialog';
import { isLeadRole } from '../../../../../data/content';

const KanbanPurge = (props: KanbanPurgeProps) => {
  const { boardKey, request, onClose } = props;
  const data = useKanbanData(boardKey);
  if (!request) return null;
  const focus = readStored(focusKey(boardKey)) ?? '';
  const focused = data.threads.some((thread) => thread.id === focus && !isLeadRole(thread.role)) ? focus : '';
  return <PurgeDialog boardKey={boardKey} data={data} thread={request.thread ?? focused} scope={request.scope} onClose={onClose} />;
};

export { KanbanPurge };
