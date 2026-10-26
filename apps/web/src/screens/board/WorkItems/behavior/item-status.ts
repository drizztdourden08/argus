/* @layer renderer-app @kind logic */
import { ITEM_STATUSES } from '../WorkItems.constants';
import type { ItemStatusKey } from '../WorkItems.type';

const isItemStatus = (value: string): value is ItemStatusKey => Object.hasOwn(ITEM_STATUSES, value);

const itemStatus = (value: string): ItemStatusKey => (isItemStatus(value) ? value : 'todo');

export { itemStatus };
