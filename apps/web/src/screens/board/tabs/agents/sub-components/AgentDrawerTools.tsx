/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import { DrawerToolbar, neighbours } from '../../../shared';
import type { AgentDrawerToolsProps } from '../drawer-props.type';

const AgentDrawerTools = (props: AgentDrawerToolsProps) => {
  const { id, agent, order, actions, wide, onWide, onOpen } = props;
  const archived = Boolean(agent.archivedAt);
  return (
    <DrawerToolbar id={id} {...neighbours(order, id)} label="Agent tools" wide={wide} onWide={onWide} onStep={onOpen}>
      <IconButton label={archived ? 'Unarchive this agent' : 'Archive this agent'} size="xs" variant="ghost" onClick={() => actions.soft(archived ? 'unarchive' : 'archive', id)}>
        <Icon name="archive" />
      </IconButton>
      <IconButton label="Delete this agent" size="xs" variant="ghost" onClick={() => actions.soft('delete', id)}>
        <Icon name="trash-2" className="agents-icon--danger" />
      </IconButton>
    </DrawerToolbar>
  );
};

export { AgentDrawerTools };
