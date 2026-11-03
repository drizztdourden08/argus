/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { Agent } from '../../../../../data/agents';
import { useCollection } from '../../../../../data/react';
import type { WithId } from '../../../../../data/react';

interface AgentDocLoad {
  agent: WithId<Agent> | null;
  agents: readonly WithId<Agent>[];
  ready: boolean;
}

const useAgentDoc = (boardKey: string, id: string): AgentDocLoad => {
  const loaded = useCollection<Agent>(boardKey, 'agents');
  return useMemo(() => {
    const agent = loaded.docs.find((doc) => doc.id === id) ?? null;
    return { agent, agents: agent ? [agent] : [], ready: loaded.ready };
  }, [loaded, id]);
};

export { useAgentDoc };
