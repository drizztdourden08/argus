/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';
import { agentTask } from './agent-task';
import { nameOf } from './name-of';
import type { WorktreeRow } from './model.type';

const ownBranch = (row: WorktreeRow): boolean => {
  const name = nameOf(row);
  return !row.branch || row.branch === name || row.branch === `worktree-${name}`;
};

const worktreeTitle = (row: WorktreeRow, agents: readonly WithId<Agent>[]): string => {
  const task = agentTask(row, agents);
  if (task) return task;
  const subject = row.subject?.trim();
  if (subject) return subject;
  return ownBranch(row) ? '' : (row.branch ?? '');
};

export { worktreeTitle };
