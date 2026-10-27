/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import type { AgentRowProps } from '../AgentsTab.type';
import { agentNow, isActive, isStale, isStalled, shortBranch } from '../model';
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

const rowClass = (props: AgentRowProps): string => {
  const { agent, now } = props;
  const flags = [isActive(agent) ? '' : 'fin', isStale(agent, now) ? 'stale' : '', isStalled(agent) ? 'stalled' : ''];
  return ['agents-row', ...flags.filter(Boolean).map((flag) => `agents-row--${flag}`)].join(' ');
};

const branchTip = (props: AgentRowProps): string => {
  const { agent } = props;
  return `${agent.branch ?? 'no branch'}${agent.worktree ? `\n${agent.worktree}` : ''}`;
};

const AgentRowMain = (props: AgentRowProps) => {
  const { id, agent, now, actions, state, runs, onOpen, onLog } = props;
  const doing = agentNow(agent);
  return (
    <Box role="row" className={rowClass(props)} data-agent-id={id} onClick={() => onOpen(id)}>
      <AgentStatus agent={agent} now={now} />
      <AgentName id={id} agent={agent} now={now} runs={runs} onOpen={onOpen} onLog={onLog} />
      <AgentModel agent={agent} />
      <AgentCell name="kind"><Text as="span" tone="dim">{agent.kind || '-'}</Text></AgentCell>
      <AgentItem item={agent.item} />
      <AgentCell name="branch" title={branchTip(props)}><Text as="span" tone="dim">{agent.branch ? shortBranch(agent.branch) : '-'}</Text></AgentCell>
      <AgentTime agent={agent} now={now} />
      <AgentCell name="now" title={doing?.text ?? agent.live?.liveNote ?? ''}>
        {isActive(agent) && agent.live ? <AgentNow agent={agent} now={now} /> : <Text as="span" tone="muted">-</Text>}
      </AgentCell>
      <AgentMessage id={id} agent={agent} open={state.openMessages.has(id)} onToggle={state.toggleMessage} />
      <AgentNote agent={agent} />
      <AgentActionButtons id={id} agent={agent} actions={actions} state={state} onOpen={onOpen} />
    </Box>
  );
};

export { AgentRowMain };
