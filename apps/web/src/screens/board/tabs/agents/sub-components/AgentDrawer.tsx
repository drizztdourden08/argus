/* @layer renderer-app @kind component */
import { RecordDrawer, WIDE_KEY, useStoredFlag } from '../../../shared';
import type { AgentDrawerProps } from '../drawer-props.type';
import { agentName } from '../model';
import { AgentDrawerBody } from './AgentDrawerBody';
import { AgentDrawerTools } from './AgentDrawerTools';
import { AgentEyebrow } from './AgentEyebrow';

const AgentDrawer = (props: AgentDrawerProps) => {
  const { boardKey, id, agent, order, now, actions, onOpen, onClose } = props;
  const [wide, setWide] = useStoredFlag(WIDE_KEY);
  const shown = agent !== null && id !== null;
  return (
    <RecordDrawer
      open={shown}
      label="Agent detail"
      wide={wide}
      eyebrow={shown && <AgentEyebrow id={id} agent={agent} now={now} />}
      title={shown ? agentName(agent, id) : ''}
      tools={shown && <AgentDrawerTools id={id} agent={agent} order={order} actions={actions} wide={wide} onWide={() => setWide(!wide)} onOpen={onOpen} />}
      onClose={onClose}
    >
      {shown && <AgentDrawerBody boardKey={boardKey} id={id} agent={agent} now={now} actions={actions} />}
    </RecordDrawer>
  );
};

export { AgentDrawer };
