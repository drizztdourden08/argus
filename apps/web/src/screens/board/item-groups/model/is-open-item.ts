/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../data/kanban';
import { CLOSED_ITEM_STATUSES } from '../item-groups.constants';

const isOpenItem = (item: ItemRef): boolean => !CLOSED_ITEM_STATUSES.includes(item.status) && !item.deletedAt && !item.archivedAt;

export { isOpenItem };
