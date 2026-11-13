/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import { formatAgo } from '../../../../../format';
import { agentItems } from './agent-items';
import { agentNow } from './agent-now';
import type { DetailLine } from './model.type';

const nowValue = (agent: Agent, now: number): string => {
  const doing = agentNow(agent);
  if (!doing) return '';
  return doing.since ? `${doing.text} (${formatAgo(doing.since, now)})` : doing.text;
};

const agentDetails = (agent: Agent, now: number): DetailLine[] => {
  const lines: DetailLine[] = [
    { id: 'now', label: 'Now', value: nowValue(agent, now) },
    { id: 'message', label: 'Last message', value: agent.live?.lastMessage ?? '' },
    { id: 'note', label: 'Note', value: agent.note ?? '' },
    { id: 'items', label: 'Items', value: agentItems(agent).join(', ') },
    { id: 'worktree', label: 'Worktree', value: agent.worktree ?? agent.where ?? '' },
  ];
  return lines.filter((line) => line.value !== '');
};

export { agentDetails };
