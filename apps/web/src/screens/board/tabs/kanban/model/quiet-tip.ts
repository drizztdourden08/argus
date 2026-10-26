/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { ago } from './ago';
import { QUIET_HOURS } from './model.constants';

const quietTip = (item: ItemRef, now: number): string =>
  `No change for ${ago(item.updatedAt, now)} (a ${item.status} item is reminded after ${QUIET_HOURS[item.status]} h). Its thread must start it, update it (a note or a status) or drop or close it.`;

export { quietTip };
