/* @layer renderer-app @kind component */
import type { RowActionsProps } from '../review-tree-props.type';
import { ActionButton } from './ActionButton';
import { MoveMenu } from './MoveMenu';
import { ScopeMenu } from './ScopeMenu';

const ChatActions = (props: RowActionsProps) => {
  const { row, account, env } = props;
  const archived = row.cells[account]?.archived === true;
  const act = (name: 'archive' | 'unarchive' | 'delete_chat') => () => env.onAct({ name, account, id: row.id, title: row.title });
  const kids = (env.scopes.get(row.id)?.length ?? 0) > 0;
  return (
    <>
      <MoveMenu {...props} />
      {archived ? <ActionButton label="Unarchive" icon="rotate-ccw" onClick={act('unarchive')} /> : <ActionButton label="Archive" icon="archive" onClick={act('archive')} />}
      {archived && <ActionButton label="Delete" icon="trash-2" danger onClick={act('delete_chat')} />}
      {kids && <ScopeMenu row={row} account={account} env={env} label="This chat with its sub-threads" />}
    </>
  );
};

export { ChatActions };
