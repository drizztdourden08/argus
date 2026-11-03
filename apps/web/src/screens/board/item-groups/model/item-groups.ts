/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../data/kanban';
import { groupRecords } from '../../../../shell';
import type { RecordGroupData } from '../../../../shell/records';
import type { ItemGroup, ItemGroups } from '../item-groups.type';
import { isOpenItem } from './is-open-item';

const wrap = <T>(known: ReadonlyMap<string, ItemRef>, groups: readonly RecordGroupData<T>[]): ItemGroup<T>[] =>
  groups.flatMap((group) => {
    const item = known.get(group.key);
    return item ? [{ item, rows: group.rows }] : [];
  });

const itemGroups = <T>(rows: readonly T[], items: readonly ItemRef[], itemOf: (row: T) => string | null | undefined): ItemGroups<T> => {
  const known = new Map(items.filter((item) => !item.deletedAt).map((item) => [item.id, item]));
  const keysOf = (row: T): string[] => {
    const id = itemOf(row);
    return id && known.has(id) ? [id] : [];
  };
  const info = (key: string) => {
    const item = known.get(key);
    return item ? { rank: item.rank ?? 0, closed: !isOpenItem(item) } : null;
  };
  const found = groupRecords(rows, keysOf, info);
  return { groups: wrap(known, found.groups), none: found.none, closed: wrap(known, found.closed) };
};

export { itemGroups };
