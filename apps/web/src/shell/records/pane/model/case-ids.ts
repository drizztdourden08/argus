/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../data/kanban';

const caseIds = (item: Pick<ItemRef, 'fields'>): string[] => {
  const raw = item.fields?.cases;
  return Array.isArray(raw) ? raw.filter((id): id is string => typeof id === 'string') : [];
};

export { caseIds };
