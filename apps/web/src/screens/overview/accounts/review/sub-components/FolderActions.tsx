/* @layer renderer-app @kind component */
import { NO_FOLDER_ID } from '../review.constants';
import type { RowActionsProps } from '../review-tree-props.type';
import { ActionButton } from './ActionButton';
import { ScopeMenu } from './ScopeMenu';

const FolderActions = (props: RowActionsProps) => {
  const { row, account, env } = props;
  const act = (name: 'rename' | 'delete_folder' | 'new_folder') => () => env.onAct({ name, account, id: row.id, title: row.title });
  const named = row.id !== NO_FOLDER_ID;
  return (
    <>
      {named && <ActionButton label="Rename" icon="pencil" onClick={act('rename')} />}
      {named && <ActionButton label="Delete folder" icon="trash-2" danger onClick={act('delete_folder')} />}
      <ActionButton label="New folder" icon="plus" onClick={act('new_folder')} />
      <ScopeMenu row={row} account={account} env={env} label="All chats in this folder" />
    </>
  );
};

export { FolderActions };
