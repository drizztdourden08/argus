/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../data/kanban';
import { FINISHED_STATUSES } from '../ThreadDrawer.constants';

const openWork = (items: readonly ItemRef[], thread: string): ItemRef[] =>
  items.filter((item) => item.thread === thread && !item.deletedAt && !item.archivedAt && !FINISHED_STATUSES.includes(item.status));

export { openWork };
