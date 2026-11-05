/* @layer renderer-app @kind logic */
import { EDIT_LIMITS } from './model.constants';

const fieldValidator = (field: 'title' | 'covers') => (value: string): string | undefined => {
  const text = value.trim();
  if (field === 'title' && !text) return 'The title cannot be empty.';
  return text.length > EDIT_LIMITS[field] ? `The ${field} is at most ${EDIT_LIMITS[field]} characters.` : undefined;
};

export { fieldValidator };
