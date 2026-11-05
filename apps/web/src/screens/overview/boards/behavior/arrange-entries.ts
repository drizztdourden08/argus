/* @layer renderer-app @kind logic */
import type { SortState } from '@drizztdourden08/tessera';
import type { BoardEntry, SortKey } from '../boards-view.type';
import { ENTRY_COMPARE } from './entry-compare.constants';
import type { EntryCompare } from './entry-compare.type';

const compareOf = (key: string | null): EntryCompare =>
  key !== null && key in ENTRY_COMPARE ? ENTRY_COMPARE[key as SortKey] : ENTRY_COMPARE.attention;

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
