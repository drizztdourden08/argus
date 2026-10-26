/* @layer renderer-app @kind component */
import type { KanbanDialogsProps } from '../kanban-props.type';
import { AddItemDialog } from './AddItemDialog';
import { PurgeDialog } from './PurgeDialog';

const KanbanDialogs = (props: KanbanDialogsProps) => {
  const { boardKey, data, focus, ops, dialog, onClose } = props;
  if (dialog?.kind === 'add') {
    return <AddItemDialog preset={dialog.preset} threads={data.threads} onCreate={(item) => void ops.create(item)} onClose={onClose} />;
  }
  if (dialog?.kind === 'purge') {
    return <PurgeDialog boardKey={boardKey} data={data} focus={focus} scope={dialog.scope} onClose={onClose} />;
  }
  return null;
};

export { KanbanDialogs };
