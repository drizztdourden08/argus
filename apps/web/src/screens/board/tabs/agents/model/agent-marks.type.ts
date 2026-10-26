/* @layer renderer-app @kind types */
import type { Agent } from '../../../../../data/agents';
import type { AgentMark } from './model.type';

type MarkRule = (agent: Agent, now: number) => AgentMark | null;

export type { MarkRule };
