/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import type { TrackColumn } from '@drizztdourden08/tessera';
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';
import type { AgentsBodyProps } from '../AgentsTab.type';
import { agentNow, agentWidths, isActive, orText, shortBranch } from '../model';
import { AgentActionButtons } from './AgentActionButtons';
import { AgentCell } from './AgentCell';
import { AgentItem } from './AgentItem';
import { AgentMessage } from './AgentMessage';
import { AgentModel } from './AgentModel';
import { AgentName } from './AgentName';
import { AgentNote } from './AgentNote';
import { AgentNow } from './AgentNow';
import { AgentStatus } from './AgentStatus';
import { AgentTime } from './AgentTime';

const branchTip = (agent: Agent): string => `${agent.branch ?? 'no branch'}${agent.worktree ? `\n${agent.worktree}` : ''}`;

const nowCell = (agent: Agent, now: number) => {
  const doing = agentNow(agent);
  return (
    <AgentCell name="now" title={doing?.text ?? agent.live?.liveNote ?? ''}>
      {isActive(agent) && agent.live ? <AgentNow agent={agent} now={now} /> : <Box as="span" className="agents-na">-</Box>}
    </AgentCell>
  );
};

const agentColumns = (props: AgentsBodyProps): TrackColumn<WithId<Agent>>[] => {
  const { now, actions, state, runs, onOpen, onLog } = props;
  return [
    { id: 'status', label: 'Status', width: agentWidths.status, cell: (row) => <AgentStatus id={row.id} agent={row} now={now} /> },
    { id: 'agent', label: 'Agent', width: agentWidths.agent, cell: (row) => <AgentName id={row.id} agent={row} now={now} runs={runs.byAgent.get(row.id) ?? []} onOpen={onOpen} onLog={onLog} /> },
    { id: 'model', label: 'Model', width: agentWidths.model, drop: 1, cell: (row) => <AgentModel agent={row} /> },
    { id: 'kind', label: 'Kind', width: agentWidths.kind, drop: 2, cell: (row) => <AgentCell name="kind">{orText(row.kind, '-')}</AgentCell> },
    { id: 'item', label: 'Item', width: agentWidths.item, drop: 2, cell: (row) => <AgentItem item={row.item} /> },
    { id: 'branch', label: 'Branch', width: agentWidths.branch, drop: 2, cell: (row) => <AgentCell name="branch" title={branchTip(row)}>{row.branch ? shortBranch(row.branch) : '-'}</AgentCell> },
    { id: 'time', label: 'Time', width: agentWidths.time, drop: 2, cell: (row) => <AgentTime agent={row} now={now} /> },
    { id: 'now', label: 'Now', width: agentWidths.now, drop: 2, cell: (row) => nowCell(row, now) },
    { id: 'message', label: 'Last message', fr: 2, drop: 2, cell: (row) => <AgentMessage id={row.id} agent={row} open={state.openMessages.has(row.id)} onToggle={state.toggleMessage} /> },
    { id: 'note', label: 'Note', fr: 1, cell: (row) => <AgentNote agent={row} /> },
    { id: 'actions', label: <Box as="span" className="visually-hidden">Actions</Box>, width: agentWidths.actions, cell: (row) => <AgentActionButtons id={row.id} agent={row} actions={actions} state={state} onOpen={onOpen} /> },
  ];
};

export { agentColumns };
