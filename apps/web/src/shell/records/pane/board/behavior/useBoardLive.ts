/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { Agent } from '../../../../../data/agents';
import { useDeps } from '../../../../../data/deps';
import type { Item } from '../../../../../data/kanban';
import { useCollection } from '../../../../../data/react';
import type { Launcher, Test } from '../../../../../data/types';
import type { Worktree } from '../../../../../data/worktrees';
import type { BoardLive } from '../board.type';

const useBoardLive = (key: string): BoardLive => {
  const items = useCollection<Item>(key, 'items');
  const agents = useCollection<Agent>(key, 'agents');
  const tests = useCollection<Test>(key, 'tests');
  const worktrees = useCollection<Worktree>(key, 'worktrees');
  const launchers = useCollection<Launcher>(key, 'launchers');
  const deps = useDeps();
  return useMemo(
    () => ({ items: items.docs, agents: agents.docs, tests: tests.docs, worktrees: worktrees.docs, launchers: launchers.docs, deps: deps.docs }),
    [items.docs, agents.docs, tests.docs, worktrees.docs, launchers.docs, deps.docs],
  );
};

export { useBoardLive };
