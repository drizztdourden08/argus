/* @layer renderer-app @kind types */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';

type AgentGroup = 'item' | 'thread' | 'status' | 'model' | 'none';

type AgentOrder = 'updated' | 'started' | 'name';

type AgentFilterKey = 'thread' | 'status' | 'model' | 'item';

interface AgentFilters {
  query: string;
  thread: string[];
  status: string[];
  model: string[];
  item: string[];
}

interface AgentViewSpec {
  filters: AgentFilters;
  order: AgentOrder;
  direction: SortDirection;
}

interface ShownAgents {
  active: WithId<Agent>[];
  done: WithId<Agent>[];
}

interface KeyGroup {
  key: string;
  label: string;
  rows: WithId<Agent>[];
}

export type { AgentFilterKey, AgentFilters, AgentGroup, AgentOrder, AgentViewSpec, KeyGroup, ShownAgents };
