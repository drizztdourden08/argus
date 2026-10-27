/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import type { AgentStopButtonProps } from '../row-props.type';

const AgentStopButton = (props: AgentStopButtonProps) => {
  const { id, agent, actions } = props;
  if (agent.status === 'working' || agent.status === 'waiting') {
    return (
      <IconButton label="Ask the lead to stop this agent" variant="ghost" size="sm" onClick={() => actions.stop(id)}>
        <Icon name="square" className="agents-icon--danger" />
      </IconButton>
    );
  }
  if (agent.status !== 'stop-requested') return null;
  return (
    <IconButton label="Cancel the stop request" variant="ghost" size="sm" onClick={() => actions.cancelStop(id)}>
      <Icon name="undo-2" className="agents-icon--accent" />
    </IconButton>
  );
};

export { AgentStopButton };
