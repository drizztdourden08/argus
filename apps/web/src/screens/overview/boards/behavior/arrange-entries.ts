/* @layer renderer-app @kind logic */
import type { SortState } from '@drizztdourden08/tessera';
import type { BoardEntry, SortKey } from '../boards-view.type';
import { attentionGroup } from '../model';
import { GROUP_ORDER } from '../model/board.constants';

type Compare = (left: BoardEntry, right: BoardEntry) => number;

const rank = (entry: BoardEntry): number => GROUP_ORDER.indexOf(attentionGroup(entry.record, entry.live));

const COMPARE: Readonly<Record<SortKey, Compare>> = {
  attention: (left, right) => rank(left) - rank(right),
  name: (left, right) => left.record.name.localeCompare(right.record.name),
  updated: (left, right) => right.updatedAt.localeCompare(left.updatedAt),
};

const compareOf = (key: string | null): Compare => COMPARE[key as SortKey] ?? COMPARE.attention;

const matches = (entry: BoardEntry, query: string): boolean => {
  const { record } = entry;
  return [record.name, record.key, record.id, record.goal].some((text) => text.toLowerCase().includes(query));
};

const arrangeEntries = (entries: readonly BoardEntry[], search: string, sort: SortState): BoardEntry[] => {
  const query = search.trim().toLowerCase();
  const compare = compareOf(sort.key);
  const sign = sort.direction === 'desc' ? -1 : 1;
  return entries.filter((entry) => matches(entry, query)).sort((left, right) => sign * compare(left, right));
};

export { arrangeEntries };
