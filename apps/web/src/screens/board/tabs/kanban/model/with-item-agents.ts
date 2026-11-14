/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import type { AgentDoc } from '../kanban.type';
import { itemAgents } from './item-agents';

const withItemAgents = (items: readonly ItemRef[], agents: readonly WithId<AgentDoc>[]): ItemRef[] =>
  items.map((item) => {
    if ((item.agents?.length ?? 0) > 0) return item;
    const linked = itemAgents(agents, item.id);
    if (linked.length === 0) return item;
    return { ...item, agents: linked.map((agent) => ({ id: agent.id, name: (agent.name?.trim() ?? '') || agent.id, status: agent.status })) };
  });

export { withItemAgents };
