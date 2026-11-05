/* @layer renderer-app @kind types */
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';

type AgentCompare = (a: WithId<Agent>, b: WithId<Agent>) => number;

export type { AgentCompare };
