/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { Item } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { groupRecords } from '../../../../../shell';
import type { RecordGroups } from '../../../../../shell/records';
import { agentItems } from './agent-items';
import { itemInfo } from './item-info';

const groupAgents = (agents: readonly WithId<Agent>[], items: readonly WithId<Item>[]): RecordGroups<WithId<Agent>> =>
  groupRecords(agents, agentItems, itemInfo(items));

export { groupAgents };
