/* @layer renderer-app @kind component */
import { DateText, ListItemRow, Tag, Text } from '@drizztdourden08/tessera';
import type { ListItemColumn } from '@drizztdourden08/tessera';
import type { Agent } from '../../../../../data/agents';
import type { AgentRowProps } from '../AgentsTab.type';
import { shortBranch } from '../model';
import { AgentActionButtons } from './AgentActionButtons';
import { AgentDetailLine } from './AgentDetailLine';
import { AgentModel } from './AgentModel';
import { AgentName } from './AgentName';
import { AgentStatus } from './AgentStatus';
import { AgentTime } from './AgentTime';

const columnsOf = (agent: Agent, now: number): ListItemColumn[] => [
  {
    primary: <AgentStatus agent={agent} />,
    secondary: agent.updatedAt ? <DateText value={agent.updatedAt} format="relative" /> : undefined,
  },
  { primary: <AgentModel agent={agent} />, secondary: agent.kind ?? '-' },
  {
    primary: agent.item ? <Tag variant="normal" color="neutral">{agent.item}</Tag> : <Text tone="muted">-</Text>,
    secondary: agent.branch ? shortBranch(agent.branch) : undefined,
  },
  {
    primary: <AgentTime agent={agent} now={now} />,
    secondary: agent.live ? `${agent.live.turns ?? 0} turns, ${agent.live.toolCalls ?? 0} tools` : undefined,
    align: 'end',
  },
];

const AgentRow = (props: AgentRowProps) => {
  const { id, agent, now, actions, onOpen } = props;
  return (
    <ListItemRow
      name={<AgentName id={id} agent={agent} now={now} />}
      meta={<AgentDetailLine agent={agent} />}
      columns={columnsOf(agent, now)}
      action={<AgentActionButtons id={id} agent={agent} actions={actions} />}
      onClick={() => onOpen(id)}
      data={{ 'data-agent-id': id }}
    />
  );
};

export { AgentRow };
