/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';

const subjectsOf = (items: readonly ItemRef[]): string[] =>
  [...new Set(items.filter((item) => !item.deletedAt).map((item) => item.subject).filter((subject): subject is string => Boolean(subject)))].sort();

export { subjectsOf };
