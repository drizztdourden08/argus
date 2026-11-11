/* @layer renderer-app @kind logic */
import type { ReviewEdit } from '../../../../../data/types';

const rowKeyOf = (edit: ReviewEdit): string => {
  if ('sessionId' in edit) return edit.sessionId;
  return `folder:${'folder' in edit ? edit.folder : edit.name}`;
};

const pendingKeys = (edits: readonly ReviewEdit[]): Set<string> => new Set(edits.map((edit) => `${edit.account}|${rowKeyOf(edit)}`));

export { pendingKeys };
