/* @layer renderer-app @kind component */
import { Box, Icon, IconButton } from '@drizztdourden08/tessera';
import { copyText } from '../../../shared';
import type { AgentActionsProps } from '../AgentsTab.type';
import { AgentCell } from './AgentCell';
import { AgentMenu } from './AgentMenu';

const StopButton = (props: AgentActionsProps) => {
  const { id, agent, actions } = props;
  if (agent.status === 'working' || agent.status === 'waiting') {
    return (
      <IconButton label="Ask the lead to stop this agent" variant="ghost" size="sm" className="agents-icon agents-icon--danger" onClick={() => actions.stop(id)}>
        <Icon name="square" />
      </IconButton>
    );
  }
  if (agent.status !== 'stop-requested') return null;
  return (
    <IconButton label="Cancel the stop request" variant="ghost" size="sm" className="agents-icon agents-icon--accent" onClick={() => actions.cancelStop(id)}>
      <Icon name="undo-2" />
    </IconButton>
  );
};

const BranchButton = (props: { branch: string | null | undefined }) => {
  const { branch } = props;
  if (!branch) return null;
  return (
    <IconButton label={`Copy the branch: ${branch}`} variant="ghost" size="sm" className="agents-icon agents-icon--info agents-actions__extra" onClick={() => copyText(branch)}>
      <Icon name="git-fork" />
    </IconButton>
  );
};

const AgentActionButtons = (props: AgentActionsProps) => {
  const { id, agent, actions, state, onOpen } = props;
  return (
    <AgentCell name="actions">
      <Box as="span" className="agents-actions" onClick={(event) => event.stopPropagation()}>
        <IconButton label="Edit the note" variant="ghost" size="sm" className="agents-icon agents-actions__extra" onClick={() => state.startEdit(id)}>
          <Icon name="pencil" />
        </IconButton>
        <StopButton {...props} />
        <BranchButton branch={agent.branch} />
        <Box as="span" className="agents-actions__menu">
          <AgentMenu id={id} agent={agent} actions={actions} onOpen={onOpen} onEditNote={() => state.startEdit(id)} />
        </Box>
      </Box>
    </AgentCell>
  );
};

export { AgentActionButtons };
