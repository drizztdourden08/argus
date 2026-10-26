/* @layer renderer-app @kind component */
import { CopyButton, Flex, Icon, IconButton } from '@drizztdourden08/tessera';
import type { AgentActionsProps } from '../AgentsTab.type';

const AgentActionButtons = (props: AgentActionsProps) => {
  const { id, agent, actions } = props;
  const canStop = agent.status === 'working' || agent.status === 'waiting';
  return (
    <Flex gap="xs" align="center">
      {canStop && (
        <IconButton label="Ask the lead to stop this agent" variant="danger" size="xs" onClick={() => actions.stop(id)}>
          <Icon name="square" />
        </IconButton>
      )}
      {agent.status === 'stop-requested' && (
        <IconButton label="Cancel the stop request" variant="info" size="xs" onClick={() => actions.cancelStop(id)}>
          <Icon name="undo-2" />
        </IconButton>
      )}
      {agent.branch && <CopyButton text={agent.branch} label={`Copy the branch: ${agent.branch}`} size="xs" variant="ghost" />}
    </Flex>
  );
};

export { AgentActionButtons };
