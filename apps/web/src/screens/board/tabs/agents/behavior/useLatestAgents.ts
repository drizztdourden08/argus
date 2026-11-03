/* @layer renderer-app @kind logic */
import { useEffect, useRef } from 'react';
import type { RefObject } from 'react';
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';

const useLatestAgents = (agents: readonly WithId<Agent>[]): RefObject<Map<string, Agent>> => {
  const latest = useRef(new Map<string, Agent>());
  useEffect(() => {
    latest.current = new Map(agents.map((agent) => [agent.id, agent]));
  }, [agents]);
  return latest;
};

export { useLatestAgents };
