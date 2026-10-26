/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';
import type { AgentFilter } from './model.type';

const haystack = (agent: WithId<Agent>): string =>
  [agent.id, agent.name, agent.task, agent.kind, agent.item, agent.model, agent.note, agent.branch, agent.thread].join(' ').toLowerCase();

const poolOf = (filter: AgentFilter, active: readonly WithId<Agent>[], done: readonly WithId<Agent>[]): readonly WithId<Agent>[] => {
  if (filter === 'running') return active;
  return filter === 'done' ? done : [...active, ...done];
};

const filterAgents = (active: readonly WithId<Agent>[], done: readonly WithId<Agent>[], filter: AgentFilter, query: string): WithId<Agent>[] => {
  const pool = poolOf(filter, active, done);
  const needle = query.trim().toLowerCase();
  return needle ? pool.filter((agent) => haystack(agent).includes(needle)) : [...pool];
};

export { filterAgents };
