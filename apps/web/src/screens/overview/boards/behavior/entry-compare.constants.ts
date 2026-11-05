/* @layer renderer-app @kind constants */
import type { BoardEntry, SortKey } from '../boards-view.type';
import type { EntryCompare } from './entry-compare.type';
import { entryRank } from './entry-rank';

const ENTRY_COMPARE: Readonly<Record<SortKey, EntryCompare>> = {
  attention: (left: BoardEntry, right: BoardEntry) => entryRank(left) - entryRank(right),
  name: (left, right) => left.record.name.localeCompare(right.record.name),
  updated: (left, right) => right.updatedAt.localeCompare(left.updatedAt),
};

export { ENTRY_COMPARE };
