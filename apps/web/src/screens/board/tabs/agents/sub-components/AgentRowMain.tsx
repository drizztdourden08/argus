/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import type { AgentRowProps } from '../row-props.type';
import { agentNow, isActive, isStale, isStalled, orText, shortBranch } from '../model';
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
      <AgentCell name="kind">{orText(agent.kind, '-')}</AgentCell>
      <AgentItem item={agent.item} />
      <AgentCell name="branch" title={branchTip(props)}>{agent.branch ? shortBranch(agent.branch) : '-'}</AgentCell>
      <AgentTime agent={agent} now={now} />
      <AgentCell name="now" title={doing?.text ?? agent.live?.liveNote ?? ''}>
        {isActive(agent) && agent.live ? <AgentNow agent={agent} now={now} /> : <Box as="span" className="agents-na">-</Box>}
      </AgentCell>
      <AgentMessage id={id} agent={agent} open={state.openMessages.has(id)} onToggle={state.toggleMessage} />
      <AgentNote agent={agent} />
      <AgentActionButtons id={id} agent={agent} actions={actions} state={state} onOpen={onOpen} />
    </Box>
  );
};

export { AgentRowMain };
