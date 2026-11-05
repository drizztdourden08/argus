/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';

const casesOf = (plan: ItemRef): string[] => {
  const raw = plan.fields?.cases;
  return Array.isArray(raw) ? raw.filter((id): id is string => typeof id === 'string') : [];
};

export { casesOf };
