/* @layer renderer-app @kind component */
import { Drawer } from '@drizztdourden08/tessera';
import { WIDE_KEY, useStoredFlag } from '../../../shared';
import type { AgentDrawerProps } from '../AgentsTab.type';
import { agentName } from '../model';
import { AgentDrawerBody } from './AgentDrawerBody';

const AgentDrawer = (props: AgentDrawerProps) => {
  const { boardKey, id, agent, order, now, actions, onOpen, onClose } = props;
  const [wide, setWide] = useStoredFlag(WIDE_KEY);
  const shown = agent !== null && id !== null;
  return (
    <Drawer open={shown} onClose={onClose} size={wide ? 'xl' : 'md'} label="Agent detail" title={shown ? agentName(agent, id) : ''} subtitle={id}>
      {shown && (
        <AgentDrawerBody
          boardKey={boardKey}
          id={id}
          agent={agent}
          order={order}
          now={now}
          actions={actions}
          wide={wide}
          onWide={() => setWide(!wide)}
          onOpen={onOpen}
        />
      )}
    </Drawer>
  );
};

export { AgentDrawer };
