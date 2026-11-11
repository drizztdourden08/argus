/* @layer renderer-app @kind logic */
import type { ReviewEdit } from '../../../../../data/types';
import type { ChatEdit, EditNames } from '../review.type';

const chatText = (edit: ChatEdit, names: EditNames): string => `"${names.titles.get(edit.sessionId) ?? edit.sessionId}"`;

const bodyOf = (edit: ReviewEdit, names: EditNames): string => {
  switch (edit.op) {
    case 'move_chat': return `Move ${chatText(edit, names)} to ${edit.folder ?? 'no folder'}`;
    case 'archive': return `Archive ${chatText(edit, names)}`;
    case 'unarchive': return `Unarchive ${chatText(edit, names)}`;
    case 'delete_chat': return `Delete ${chatText(edit, names)}`;
    case 'rename_folder': return `Rename folder "${edit.folder}" to "${edit.name}"`;
    case 'delete_folder': return `Delete folder "${edit.folder}"`;
    case 'create_folder': return `New folder "${edit.name}"`;
  }
};

const editLabel = (edit: ReviewEdit, names: EditNames): string => `${bodyOf(edit, names)} on ${names.accounts.get(edit.account) ?? edit.account}`;

export { editLabel };
