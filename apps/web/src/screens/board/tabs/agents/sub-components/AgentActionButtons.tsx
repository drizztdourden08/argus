/* @layer renderer-app @kind component */
import { Box, Icon, IconButton } from '@drizztdourden08/tessera';
import type { AgentActionsProps } from '../row-props.type';
import { AgentBranchButton } from './AgentBranchButton';
import { AgentCell } from './AgentCell';
import { AgentMenu } from './AgentMenu';
import { AgentStopButton } from './AgentStopButton';

const AgentActionButtons = (props: AgentActionsProps) => {
  const { id, agent, actions, state, onOpen } = props;
  return (
    <AgentCell name="actions">
      <Box as="span" className="agents-actions" onClick={(event) => event.stopPropagation()}>
        <IconButton label="Edit the note" variant="ghost" size="sm" className="agents-actions__extra" onClick={() => state.startEdit(id)}>
          <Icon name="pencil" />
        </IconButton>
        <AgentStopButton id={id} agent={agent} actions={actions} />
        <AgentBranchButton branch={agent.branch} />
        <Box as="span" className="agents-actions__menu">
          <AgentMenu id={id} agent={agent} actions={actions} onOpen={onOpen} onEditNote={() => state.startEdit(id)} />
        </Box>
      </Box>
    </AgentCell>
  );
};

export { AgentActionButtons };
