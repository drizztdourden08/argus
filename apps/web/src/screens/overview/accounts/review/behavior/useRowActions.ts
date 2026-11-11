/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import type { DialogState, EditStager, RowAction, RowActFn } from '../review.type';

const useRowActions = (stage: EditStager, dialog: DialogState): RowActFn =>
  useCallback((action: RowAction) => {
    const { name, account, id, title } = action;
    if (name === 'rename') dialog.open({ kind: 'rename', account, folder: title });
    else if (name === 'new_folder') dialog.open({ kind: 'new', account });
    else if (name === 'move') dialog.open({ kind: 'move', account, ids: [id] });
    else if (name === 'delete_folder') stage([{ op: 'delete_folder', account, folder: title }]);
    else stage([{ op: name, account, sessionId: id }]);
  }, [stage, dialog]);

export { useRowActions };
