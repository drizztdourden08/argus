/* @layer renderer-app @kind logic */
import type { ItemDoc, TestDoc } from './model.type';

const subjectsOf = (items: readonly ItemDoc[], tests: readonly TestDoc[]): string[] => {
  const all = [...items, ...tests].filter((doc) => !doc.deletedAt).map((doc) => doc.subject);
  return [...new Set(all.filter((subject): subject is string => Boolean(subject)))].sort();
};

export { subjectsOf };
