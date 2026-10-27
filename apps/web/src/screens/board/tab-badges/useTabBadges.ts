/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { Agent } from '../../../data/agents';
import { useCollection } from '../../../data/react';
import type { Launcher } from '../../../data/types';
import type { Worktree } from '../../../data/worktrees';
import type { BoardWork } from '../feed-model';
import { useTestsBadge } from '../tabs/tests';
import { useMinute } from './behavior/useMinute';
import { boardBadges } from './tab-badges';
import type { BoardBadges } from './tab-badges.type';

const useTabBadges = (boardKey: string, work: BoardWork): BoardBadges => {
  const agents = useCollection<Agent>(boardKey, 'agents');
  const launchers = useCollection<Launcher>(boardKey, 'launchers');
  const worktrees = useCollection<Worktree>(boardKey, 'worktrees');
  const toRun = useTestsBadge(boardKey);
  const now = useMinute();
  return useMemo(
    () => boardBadges({ work, agents: agents.docs, launchers: launchers.docs, worktrees: worktrees.docs, toRun, now }),
    [work, agents.docs, launchers.docs, worktrees.docs, toRun, now],
  );
};

export { useTabBadges };
