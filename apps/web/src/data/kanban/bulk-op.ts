/* @layer renderer-app @kind logic */
import type { DocData } from '../api';
import { postEntry } from '../posts';
import { ownerEntry } from '../posts/owner-entry';
import { BULK_CHUNK } from './kanban.constants';
import type { SentBulk } from './kanban.type';

const chunksOf = (ops: readonly DocData[]): DocData[][] =>
  Array.from({ length: Math.ceil(ops.length / BULK_CHUNK) }, (_, index) => ops.slice(index * BULK_CHUNK, (index + 1) * BULK_CHUNK));

const bulkOp = async (key: string, ops: readonly DocData[], undoes: readonly string[] = []): Promise<SentBulk> => {
  const entries = chunksOf(ops).map((chunk, index) => ownerEntry({ kind: 'bulk', ops: chunk, undoes: undoes[index] }));
  const results = await Promise.all(entries.map((entry) => postEntry(key, entry)));
  return { ids: entries.map((entry) => String(entry.id)), results };
};

export { bulkOp };
