/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { Agent, ProjectThread } from '../../../../../data/agents';
import type { Item } from '../../../../../data/kanban';
import { useCollection } from '../../../../../data/react';
import type { RunDoc } from '../../../../../data/types';
import type { AgentsView } from '../AgentsTab.type';
import { agentsView, finishedCount, isActive, sortActive, sortDone } from '../model';
import type { AgentViewSpec } from '../model';

const isShown = (agent: Agent): boolean => !agent.deletedAt && !agent.archivedAt;

const flagsOf = (agent: Agent): string => `${agent.idleSince ?? ''}|${agent.orphanSince ?? ''}|${agent.status}`;

const useAgentsView = (boardKey: string, spec: AgentViewSpec): AgentsView => {
  const agents = useCollection<Agent>(boardKey, 'agents');
  const threads = useCollection<ProjectThread>(boardKey, 'threads');
  const runs = useCollection<RunDoc>(boardKey, 'runs');
  const items = useCollection<Item>(boardKey, 'items');
  const base = useMemo(() => {
    const shown = agents.docs.filter(isShown);
    const active = sortActive(shown.filter(isActive));
    const done = sortDone(shown.filter((agent) => !isActive(agent)));
    return {
      ready: agents.ready && threads.ready,
      active,
      done,
      all: [...active, ...done],
      shown: { active, done },
      items: items.docs.filter((item) => !item.deletedAt),
      threads: threads.docs,
      finishedCount: finishedCount(done, runs.docs),
      flagSignature: active.map((agent) => `${agent.id}:${flagsOf(agent)}`).join(','),
    };
  }, [agents, threads, runs, items]);
  return { ...base, shown: agentsView(base.shown, spec) };
};

export { useAgentsView };
