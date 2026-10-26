/* @layer renderer-app @kind logic */
import type { DocData } from '../api';
import { EMPTY_COLLECTION } from './stream.constants';
import type { AppliedChanges, ChangesEvent, Collections, WireChange } from './stream.type';

const withChange = (docs: ReadonlyMap<string, DocData>, change: WireChange): Map<string, DocData> => {
  const next = new Map(docs);
  if (change.op === 'delete') next.delete(change.id);
  else next.set(change.id, change.data ?? {});
  return next;
};

const applyChanges = (collections: Collections, event: ChangesEvent): AppliedChanges => {
  const next = new Map(collections);
  const touched = new Set<string>();
  for (const change of event.changes) {
    const current = next.get(change.coll) ?? EMPTY_COLLECTION;
    next.set(change.coll, { ready: current.ready, docs: withChange(current.docs, change) });
    if (current.ready) touched.add(change.coll);
  }
  return { collections: next, touched: [...touched] };
};

export { applyChanges };
