/* @layer renderer-app @kind logic */
import type { ReviewEdit } from '../../../../../data/types';

const editSlot = (edit: ReviewEdit): string => {
  switch (edit.op) {
    case 'move_chat': return `move|${edit.account}|${edit.sessionId}`;
    case 'archive':
    case 'unarchive': return `archived|${edit.account}|${edit.sessionId}`;
    case 'delete_chat': return `delete|${edit.account}|${edit.sessionId}`;
    case 'rename_folder': return `rename|${edit.account}|${edit.folder}`;
    case 'delete_folder': return `drop|${edit.account}|${edit.folder}`;
    case 'create_folder': return `new|${edit.account}|${edit.name}`;
  }
};

export { editSlot };
