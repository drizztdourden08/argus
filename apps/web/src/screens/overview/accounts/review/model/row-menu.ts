/* @layer renderer-app @kind logic */
import type { MenuGroup, MenuItem } from '@drizztdourden08/tessera';
import type { ReviewCell, ReviewRow } from '../../../../../data/types';
import { NO_FOLDER_ID } from '../review.constants';
import type { MenuContext, RowActFn, RowActionName } from '../review.type';

const entry = (ctx: MenuContext, name: RowActionName, label: string, icon: MenuItem['icon']): MenuItem => ({
  id: name,
  label,
  icon,
  tone: name === 'delete_folder' || name === 'delete_chat' ? 'danger' : undefined,
  onSelect: () => ctx.act({ name, account: ctx.account, id: ctx.row.id, title: ctx.row.title }),
});

const folderItems = (ctx: MenuContext): MenuItem[] => {
  const own = ctx.row.id === NO_FOLDER_ID ? [] : [entry(ctx, 'rename', 'Rename', 'pencil'), entry(ctx, 'delete_folder', 'Delete folder', 'trash-2')];
  return [...own, entry(ctx, 'new_folder', 'New folder', 'plus')];
};

const chatItems = (ctx: MenuContext, cell: ReviewCell): MenuItem[] => {
  const archived = cell.archived === true;
  const items = [entry(ctx, 'move', 'Move to folder', 'folder'), archived ? entry(ctx, 'unarchive', 'Unarchive', 'rotate-ccw') : entry(ctx, 'archive', 'Archive', 'archive')];
  return archived ? [...items, entry(ctx, 'delete_chat', 'Delete', 'trash-2')] : items;
};

const rowMenu = (row: ReviewRow, cell: ReviewCell, account: string, act: RowActFn): MenuGroup[] => {
  const ctx: MenuContext = { row, account, act };
  return [{ id: row.id, items: row.kind === 'folder' ? folderItems(ctx) : chatItems(ctx, cell) }];
};

export { rowMenu };
