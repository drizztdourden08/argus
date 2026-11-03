/* @layer renderer-app @kind logic */
import type { SelectOption } from '@drizztdourden08/tessera';
import type { Item } from '../../../../data/kanban';
import type { WithId } from '../../../../data/react';
import { CLOSED_STATUSES } from '../../tabs/kanban/model';
import { ITEM_LABEL_CHARS, NO_CHOICE } from '../owner-links.constants';
import { DEFAULT_TYPE } from './open-item-options.constants';

const isOpen = (item: WithId<Item>): boolean => !item.deletedAt && !CLOSED_STATUSES.includes(item.status);

const canServe = (item: WithId<Item>, serves: readonly string[] | undefined): boolean => !serves || serves.includes(item.type ?? DEFAULT_TYPE);

const openItemOptions = (items: readonly WithId<Item>[], serves: readonly string[] | undefined, placeholder: string): SelectOption[] => {
  const options = items
    .filter((item) => isOpen(item) && canServe(item, serves))
    .map((item) => ({ value: item.id, label: `${item.id} ${String(item.title).slice(0, ITEM_LABEL_CHARS)}` }));
  return [{ value: NO_CHOICE, label: placeholder }, ...options];
};

export { openItemOptions };
