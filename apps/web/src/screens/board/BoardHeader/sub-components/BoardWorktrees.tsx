/* @layer renderer-app @kind component */
import { useDoc } from '../../../../data/react';
import type { WorktreesMeta } from '../../../../data/worktrees';
import { WorktreeCounter } from '../../../../shell';
import type { BoardWorktreesProps } from '../BoardHeader.type';

const BoardWorktrees = (props: BoardWorktreesProps) => {
  const { doc } = useDoc<WorktreesMeta>(props.boardKey, 'meta', 'worktrees');
  if (typeof doc?.listed !== 'number' || typeof doc.runningAgents !== 'number') return null;
  return <WorktreeCounter listed={doc.listed} running={doc.runningAgents} />;
};

export { BoardWorktrees };
