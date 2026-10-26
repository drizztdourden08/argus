/* @layer renderer-app @kind logic */
import { LIMITS } from './model.constants';
import type { EditableField } from './model.type';

const text = (limit: number): string => `At most ${limit} characters.`;

const textProblem = (field: EditableField, value: string): string | null => {
  const trimmed = value.trim();
  if ((field === 'title' || field === 'todo') && !trimmed) return 'That cannot be empty.';
  if (field === 'criterion' && !trimmed) return 'A criterion cannot be empty.';
  return trimmed.length > LIMITS[field] ? text(LIMITS[field]) : null;
};

export { textProblem };
