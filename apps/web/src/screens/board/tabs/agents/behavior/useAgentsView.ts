/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { Agent, ProjectThread } from '../../../../../data/agents';
import type { Item } from '../../../../../data/kanban';
import { useCollection } from '../../../../../data/react';
import { HIDDEN_KEY } from '../../../BoardMore';
import { useStoredFlag } from '../../../shared';
import type { RunDoc } from '../../../../../data/types';
import type { AgentsView } from '../AgentsTab.type';
import { chatThreads, findLead, finishedCount, isActive, sortActive, sortDone } from '../model';

const isShown = (agent: Agent): boolean => !agent.deletedAt && !agent.archivedAt;

const flagsOf = (agent: Agent): string => `${agent.idleSince ?? ''}|${agent.orphanSince ?? ''}|${agent.status}`;

const useAgentsView = (boardKey: string): AgentsView => {
  const agents = useCollection<Agent>(boardKey, 'agents');
  const threads = useCollection<ProjectThread>(boardKey, 'threads');
  const runs = useCollection<RunDoc>(boardKey, 'runs');
  const items = useCollection<Item>(boardKey, 'items');
  const [showHidden] = useStoredFlag(HIDDEN_KEY);
  return useMemo(() => {
    const shown = agents.docs.filter(isShown);
    const active = sortActive(shown.filter(isActive));
    const done = sortDone(shown.filter((agent) => !isActive(agent)));
    return {
      ready: agents.ready && threads.ready,
      active,
      done,
      all: [...active, ...done],
      items: items.docs.filter((item) => !item.deletedAt),
      threads: threads.docs,
      finishedCount: finishedCount(done, runs.docs),
      lead: findLead(threads.docs),
      chats: chatThreads(threads.docs, showHidden),
      flagSignature: active.map((agent) => `${agent.id}:${flagsOf(agent)}`).join(','),
    };
  }, [agents, threads, runs, items, showHidden]);
};

export { useAgentsView };
