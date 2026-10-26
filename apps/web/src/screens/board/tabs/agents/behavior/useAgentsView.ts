/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { Agent, ProjectThread } from '../../../../../data/agents';
import { useCollection } from '../../../../../data/react';
import type { AgentsView } from '../AgentsTab.type';
import { chatThreads, findLead, isActive, sortActive, sortDone } from '../model';

const isShown = (agent: Agent): boolean => !agent.deletedAt && !agent.archivedAt;

const flagsOf = (agent: Agent): string => `${agent.idleSince ?? ''}|${agent.orphanSince ?? ''}|${agent.status}`;

const useAgentsView = (boardKey: string): AgentsView => {
  const agents = useCollection<Agent>(boardKey, 'agents');
  const threads = useCollection<ProjectThread>(boardKey, 'threads');
  return useMemo(() => {
    const shown = agents.docs.filter(isShown);
    const active = sortActive(shown.filter(isActive));
    return {
      ready: agents.ready && threads.ready,
      active,
      done: sortDone(shown.filter((agent) => !isActive(agent))),
      lead: findLead(threads.docs),
      chats: chatThreads(threads.docs),
      flagSignature: active.map((agent) => `${agent.id}:${flagsOf(agent)}`).join(','),
    };
  }, [agents, threads]);
};

export { useAgentsView };
