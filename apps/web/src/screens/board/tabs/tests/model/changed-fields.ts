/* @layer renderer-app @kind logic */
import type { EditChange, TestDoc } from './model.type';

const FIELDS: (keyof EditChange)[] = ['title', 'covers', 'mode'];

const changedFields = (test: TestDoc, draft: EditChange): Partial<EditChange> => {
  const changed: Partial<EditChange> = {};
  for (const field of FIELDS) {
    const next = draft[field].trim();
    if (next !== (test[field] ?? '')) changed[field] = next;
  }
  return changed;
};

export { changedFields };
