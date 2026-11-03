/* @layer renderer-app @kind logic */
import { NO_PARENT, NO_PARENT_TITLE, OWNER, OWNER_TITLE } from './model.constants';
import type { ColumnsInput } from './model.type';

const columnTitle = (key: string, input: ColumnsInput): string => {
  if (input.group === 'assignee' && key === OWNER) return OWNER_TITLE;
  if (input.group !== 'parent') return key;
  return key === NO_PARENT ? NO_PARENT_TITLE : `${key} ${input.parents.get(key) ?? ''}`.trim();
};

export { columnTitle };
