/* @layer renderer-app @kind types */
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';

interface AgentDocLoad {
  agent: WithId<Agent> | null;
  agents: readonly WithId<Agent>[];
  ready: boolean;
}

export type { AgentDocLoad };
