/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';
import { agentItems } from './agent-items';
import type { AgentFilters } from './agents-view.type';
import { modelFamily } from './model-family';

const passes = (picked: readonly string[], value: string): boolean => picked.length === 0 || picked.includes(value);

const textOf = (agent: WithId<Agent>): string => `${agent.id} ${agent.name ?? ''} ${agent.note ?? ''}`.toLowerCase();

const matchAgent = (agent: WithId<Agent>, filters: AgentFilters): boolean => {
  const query = filters.query.trim().toLowerCase();
  if (query && !textOf(agent).includes(query)) return false;
  if (!passes(filters.thread, agent.thread ?? '')) return false;
  if (!passes(filters.status, agent.status)) return false;
  if (!passes(filters.model, modelFamily(agent.model ?? ''))) return false;
  return filters.item.length === 0 || agentItems(agent).some((item) => filters.item.includes(item));
};

export { matchAgent };
