/* @layer renderer-app @kind logic */
import type { Item } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import type { GroupInfo } from '../../../../../shell/records';
import { CLOSED_STATUSES } from '../../kanban/model';
import { NO_RANK } from './model.constants';

const itemInfo = (items: readonly WithId<Item>[]): ((key: string) => GroupInfo | null) => {
  const byId = new Map(items.map((item) => [item.id, item]));
  return (key) => {
    const item = byId.get(key);
    return item ? { rank: item.rank ?? NO_RANK, closed: CLOSED_STATUSES.includes(item.status) } : null;
  };
};

export { itemInfo };
