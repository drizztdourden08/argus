/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';
import type { WorktreesMeta } from '../../../../../data/worktrees';
import { agentTask } from './agent-task';
import { nameOf } from './name-of';
import { relativePath } from './relative-path';
import type { WorktreeRow } from './model.type';

const ownBranch = (row: WorktreeRow): boolean => {
  const name = nameOf(row);
  return !row.branch || row.branch === name || row.branch === `worktree-${name}`;
};

const worktreeTitle = (row: WorktreeRow, agents: readonly WithId<Agent>[], meta: WorktreesMeta | null): string => {
  const task = agentTask(row, agents);
  if (task) return task;
  if (row.branch && !ownBranch(row)) return row.branch;
  return relativePath(row, meta);
};

export { worktreeTitle };
