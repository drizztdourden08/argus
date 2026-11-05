/* @layer renderer-app @kind logic */
import { EDIT_LIMITS } from './model.constants';
import type { EditChange } from './model.type';

const edit = (field: keyof EditChange, draft: EditChange): string | null => {
  const text = draft[field].trim();
  if (field === 'title' && !text) return 'The title cannot be empty.';
  return text.length > EDIT_LIMITS[field] ? `The ${field} is at most ${EDIT_LIMITS[field]} characters.` : null;
};

const editProblem = (draft: EditChange): string | null => edit('title', draft) ?? edit('covers', draft);

export { editProblem };
