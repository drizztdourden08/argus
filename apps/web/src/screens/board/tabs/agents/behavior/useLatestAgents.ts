/* @layer renderer-app @kind logic */
import { useEffect, useRef } from 'react';
import type { RefObject } from 'react';
import type { Agent } from '../../../../../data/agents';
import type { AgentsView } from '../AgentsTab.type';

const useLatestAgents = (view: AgentsView): RefObject<Map<string, Agent>> => {
  const latest = useRef(new Map<string, Agent>());
  useEffect(() => {
    latest.current = new Map([...view.active, ...view.done].map((agent) => [agent.id, agent]));
  }, [view]);
  return latest;
};

export { useLatestAgents };
