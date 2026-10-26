/* @layer renderer-app @kind component */
import { DropdownMenu } from '@drizztdourden08/tessera';
import type { BoardMoreProps } from './BoardMore.type';

const BoardMore = (props: BoardMoreProps) => {
  const { trashCount, onTrash, onPurge } = props;
  return (
    <DropdownMenu
      label="More"
      trigger={{ label: 'More', icon: 'ellipsis', iconOnly: true }}
      groups={[
        {
          id: 'board',
          items: [
            { id: 'trash', label: `Trash (${trashCount})`, onSelect: onTrash },
            { id: 'purge', label: 'Purge threads and records...', tone: 'danger', onSelect: onPurge },
          ],
        },
      ]}
      variant="ghost"
      size="sm"
      align="end"
    />
  );
};

export { BoardMore };
