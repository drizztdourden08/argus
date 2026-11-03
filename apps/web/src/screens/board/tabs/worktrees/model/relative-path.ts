/* @layer renderer-app @kind logic */
import type { WorktreesMeta } from '../../../../../data/worktrees';
import { slashed } from './slashed';
import type { WorktreeRow } from './model.type';

const relativePath = (row: WorktreeRow, meta: WorktreesMeta | null): string => {
  const path = slashed(row.path);
  const rule = meta?.rules?.find((entry) => entry.id === row.rule);
  const root = rule ? slashed(rule.path) : '';
  if (root && path.toLowerCase().startsWith(`${root.toLowerCase()}/`)) return path.slice(root.length + 1);
  return path;
};

export { relativePath };
