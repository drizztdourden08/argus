/* @layer renderer-app @kind types */
import type { WithId } from '../../../../data/react';
import type { RunEntry } from '../../../../data/runs';
import type { Worktree } from '../../../../data/worktrees';
import type { LauncherView } from '../../../../screens/board/tabs/builds';

interface Served {
  ready: boolean;
  builds: LauncherView[];
  worktrees: WithId<Worktree>[];
  runs: RunEntry[];
}

interface ServedWorktreesProps {
  rows: readonly WithId<Worktree>[];
}

interface WorktreeLineProps {
  row: WithId<Worktree>;
}

export type { Served, ServedWorktreesProps, WorktreeLineProps };
