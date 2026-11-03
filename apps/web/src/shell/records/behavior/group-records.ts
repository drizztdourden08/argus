/* @layer renderer-app @kind logic */
import type { GroupInfo, RecordGroupData, RecordGroups } from '../record-rows.type';
import { UNRANKED } from '../records.constants';

const byRank = (info: (key: string) => GroupInfo | null) => (a: RecordGroupData<unknown>, b: RecordGroupData<unknown>): number =>
  (info(a.key)?.rank ?? UNRANKED) - (info(b.key)?.rank ?? UNRANKED) || a.key.localeCompare(b.key);

const groupRecords = <T>(rows: readonly T[], keysOf: (row: T) => readonly string[], info: (key: string) => GroupInfo | null): RecordGroups<T> => {
  const found = new Map<string, T[]>();
  const none: T[] = [];
  for (const row of rows) {
    const keys = keysOf(row);
    if (keys.length === 0) none.push(row);
    for (const key of keys) found.set(key, [...(found.get(key) ?? []), row]);
  }
  const all = [...found].map(([key, grouped]): RecordGroupData<T> => ({ key, rows: grouped })).sort(byRank(info));
  return { groups: all.filter((group) => !info(group.key)?.closed), none, closed: all.filter((group) => info(group.key)?.closed) };
};

export { groupRecords };
