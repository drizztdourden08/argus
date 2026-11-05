/* @layer renderer-app @kind constants */
import type { AgentFilters, AgentGroup, AgentOrder } from './agents-view.type';
import { MODEL_FAMILIES, STATUS_LOOKS } from './model.constants';

const AGENTS_TAB = 'agents';

const AGENT_FILTERS: AgentFilters = { query: '', thread: [], status: [], model: [], item: [] };

const AGENT_GROUP_IDS: readonly AgentGroup[] = ['item', 'thread', 'status', 'model', 'none'];

const AGENT_ORDER_IDS: readonly AgentOrder[] = ['updated', 'started', 'name'];

const AGENT_GROUPS: readonly { id: AgentGroup; label: string }[] = [
  { id: 'item', label: 'Item' },
  { id: 'thread', label: 'Thread' },
  { id: 'status', label: 'Status' },
  { id: 'model', label: 'Model' },
  { id: 'none', label: 'None' },
];

const AGENT_ORDERS: readonly { id: AgentOrder; label: string }[] = [
  { id: 'updated', label: 'Updated' },
  { id: 'started', label: 'Started' },
  { id: 'name', label: 'Name' },
];

const STATUS_CHOICES = Object.entries(STATUS_LOOKS).map(([value, look]) => ({ value, label: look.label }));

const MODEL_CHOICES = [...MODEL_FAMILIES, 'other'].map((value) => ({ value, label: `${value.charAt(0).toUpperCase()}${value.slice(1)}` }));

const NONE_LABELS: Readonly<Record<AgentGroup, string>> = {
  item: 'No item',
  thread: 'No thread',
  status: 'No status',
  model: 'No model',
  none: 'All',
};

const SEARCH_PLACEHOLDER = 'Search id, name or note';

export { AGENT_FILTERS, AGENT_GROUP_IDS, AGENT_GROUPS, AGENT_ORDER_IDS, AGENT_ORDERS, AGENTS_TAB, MODEL_CHOICES, NONE_LABELS, SEARCH_PLACEHOLDER, STATUS_CHOICES };
