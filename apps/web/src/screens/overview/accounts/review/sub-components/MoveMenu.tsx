/* @layer renderer-app @kind component */
import { DropdownMenu } from '@drizztdourden08/tessera';
import { moveItems } from '../model';
import type { RowActionsProps } from '../review-tree-props.type';
import { RowTip } from './RowTip';

const MoveMenu = (props: RowActionsProps) => {
  const { row, account, env } = props;
  const move = (folder: string | null) => env.onStage([{ op: 'move_chat', account, sessionId: row.id, folder }]);
  return (
    <RowTip label="Move to folder">
      <DropdownMenu
        variant="ghost"
        size="xs"
        trigger={{ label: 'Move to folder', icon: 'folder', iconOnly: true }}
        groups={[{ id: 'folders', items: moveItems(env.folders.get(account) ?? [], move) }]}
      />
    </RowTip>
  );
};

export { MoveMenu };
