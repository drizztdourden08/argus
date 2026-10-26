/* @layer renderer-app @kind component */
import { Drawer } from '@drizztdourden08/tessera';
import type { AgentDrawerProps } from '../AgentsTab.type';
import { agentName } from '../model';
import { AgentActionButtons } from './AgentActionButtons';
import { AgentDrawerBody } from './AgentDrawerBody';

const AgentDrawer = (props: AgentDrawerProps) => {
  const { id, agent, now, actions, onClose } = props;
  const shown = agent !== null && id !== null;
  return (
    <Drawer
      open={shown}
      onClose={onClose}
      label="Agent detail"
      title={shown ? agentName(agent, id) : ''}
      subtitle={id}
      actions={shown ? <AgentActionButtons id={id} agent={agent} actions={actions} /> : null}
    >
      {shown && <AgentDrawerBody agent={agent} now={now} />}
    </Drawer>
  );
};

export { AgentDrawer };
