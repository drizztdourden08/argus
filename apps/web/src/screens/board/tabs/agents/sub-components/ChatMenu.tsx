/* @layer renderer-app @kind component */
import { DropdownMenu } from '@drizztdourden08/tessera';
import { copyText } from '../../../shared';
import type { ChatMenuProps } from '../AgentsTab.type';

const ChatMenu = (props: ChatMenuProps) => {
  const { id, onPurge } = props;
  const label = `More actions for ${id}`;
  return (
    <DropdownMenu
      label={label}
      trigger={{ label, icon: 'ellipsis', iconOnly: true }}
      groups={[
        {
          id: 'chat',
          items: [
            { id: 'purge', label: 'Purge...', tone: 'danger', onSelect: () => onPurge(id) },
            { id: 'copy', label: 'Copy id', onSelect: () => copyText(id) },
          ],
        },
      ]}
      variant="ghost"
      size="xs"
      align="end"
    />
  );
};

export { ChatMenu };
