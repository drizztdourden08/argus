/* @layer renderer-app @kind logic */
import { useCallback, useMemo } from 'react';
import type { Agent } from '../../../data/agents';
import { useCollection } from '../../../data/react';
import { readBoardReleases, REMOTE_POLL_MS, useRemoteRead } from '../../../data/remote';
import type { Launcher, PullRequest } from '../../../data/types';
import type { Worktree } from '../../../data/worktrees';
import type { BoardWork } from '../feed-model';
import { useTestsBadge } from '../tabs/tests/behavior/useTestsBadge';
import { useMinute } from './behavior/useMinute';
import { boardBadges } from './tab-badges';
import type { BoardBadges } from './tab-badges.type';

const useTabBadges = (boardKey: string, work: BoardWork): BoardBadges => {
  const agents = useCollection<Agent>(boardKey, 'agents');
  const launchers = useCollection<Launcher>(boardKey, 'launchers');
  const worktrees = useCollection<Worktree>(boardKey, 'worktrees');
  const prs = useCollection<PullRequest>(boardKey, 'prs');
  const load = useCallback(() => readBoardReleases(boardKey), [boardKey]);
  const releases = useRemoteRead(load, REMOTE_POLL_MS);
  const tests = useTestsBadge(boardKey);
  const now = useMinute();
  return useMemo(
    () => boardBadges({ work, agents: agents.docs, launchers: launchers.docs, worktrees: worktrees.docs, prs: prs.docs, channels: releases.data ?? [], tests, now }),
    [work, agents.docs, launchers.docs, worktrees.docs, prs.docs, releases.data, tests, now],
  );
};

export { useTabBadges };
