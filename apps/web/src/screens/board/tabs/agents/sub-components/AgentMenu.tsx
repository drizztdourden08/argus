/* @layer renderer-app @kind component */
import { DropdownMenu } from '@drizztdourden08/tessera';
import type { MenuItem } from '@drizztdourden08/tessera';
import { copyText } from '../../../shared';
import type { AgentMenuProps } from '../row-props.type';

const stopItems = (props: AgentMenuProps): MenuItem[] => {
  const { id, agent, actions } = props;
  if (agent.status === 'working' || agent.status === 'waiting') return [{ id: 'stop', label: 'Ask to stop', tone: 'danger', onSelect: () => actions.stop(id) }];
  return agent.status === 'stop-requested' ? [{ id: 'unstop', label: 'Cancel the stop request', onSelect: () => actions.cancelStop(id) }] : [];
};

const branchItems = (props: AgentMenuProps): MenuItem[] => {
  const { branch } = props.agent;
  return branch ? [{ id: 'copy-branch', label: 'Copy branch', onSelect: () => copyText(branch) }] : [];
};

const archiveItem = (props: AgentMenuProps): MenuItem => {
  const { id, agent, actions } = props;
  return agent.archivedAt
    ? { id: 'unarchive', label: 'Unarchive', onSelect: () => actions.soft('unarchive', id) }
    : { id: 'archive', label: 'Archive', onSelect: () => actions.soft('archive', id) };
};

const AgentMenu = (props: AgentMenuProps) => {
  const { id, actions, onOpen, onEditNote } = props;
  const label = `More actions for ${id}`;
  return (
    <DropdownMenu
      label={label}
      trigger={{ label, icon: 'ellipsis-vertical', iconOnly: true }}
      groups={[
        {
          id: 'agent',
          label: id,
          items: [
            { id: 'open', label: 'Open and edit', onSelect: () => onOpen(id) },
            { id: 'note', label: 'Edit note', onSelect: onEditNote },
            ...stopItems(props),
            ...branchItems(props),
            archiveItem(props),
            { id: 'delete', label: 'Delete', tone: 'danger', onSelect: () => actions.soft('delete', id) },
            { id: 'copy', label: 'Copy id', onSelect: () => copyText(id) },
          ],
        },
      ]}
      variant="ghost"
      size="sm"
      align="end"
      itemSize="roomy"
    />
  );
};

export { AgentMenu };
