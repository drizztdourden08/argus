/* @layer renderer-app @kind component */
import { EmptyState } from '@drizztdourden08/tessera';
import { NO_ITEMS_TEXT } from '../kanban.constants';
import type { KanbanBodyProps } from '../kanban-props.type';
import { KanbanColumns } from './KanbanColumns';
import { TrashView } from './TrashView';

const KanbanBody = (props: KanbanBodyProps) => {
  const { view, state, threads, now, selection, ops, trash, onOpen, onAdd, onPurge } = props;
  if (trash.open) {
    return <TrashView rows={trash.rows} now={now} selection={trash.selection} bulk={trash.bulk} onPurge={onPurge} />;
  }
  if (view.columns.length === 0) return <EmptyState message={NO_ITEMS_TEXT} />;
  return <KanbanColumns view={view} state={state} threads={threads} now={now} selection={selection} ops={ops} onOpen={onOpen} onAdd={onAdd} />;
};

export { KanbanBody };
