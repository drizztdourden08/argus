/* @layer renderer-app @kind logic */
import { postEntry } from '../posts';
import { BULK_CHUNK } from './agents.constants';
import { ownerEntry } from '../posts';
import type { BulkKind, BulkSent } from './agents.type';

const chunksOf = (ids: readonly string[]): string[][] =>
  Array.from({ length: Math.ceil(ids.length / BULK_CHUNK) }, (_, index) => ids.slice(index * BULK_CHUNK, (index + 1) * BULK_CHUNK));

const bulkAgents = async (key: string, kind: BulkKind, ids: readonly string[], undoes: readonly string[] = []): Promise<BulkSent> => {
  const entries = chunksOf(ids).map((chunk, index) => ownerEntry({
    kind: 'bulk',
    ops: chunk.map((id) => ({ kind, target: { c: 'agents', id } })),
    undoes: undoes[index],
  }));
  const results = await Promise.all(entries.map((entry) => postEntry(key, entry)));
  return { entryIds: entries.map((entry) => String(entry.id)), results };
};

export { bulkAgents };
