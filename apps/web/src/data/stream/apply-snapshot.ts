/* @layer renderer-app @kind logic */
import { docData } from './doc-data';
import type { Collections, SnapshotEvent } from './stream.type';

const applySnapshot = (collections: Collections, event: SnapshotEvent): Collections => {
  const docs = new Map(event.docs.map((doc) => [doc.id, docData(doc)]));
  return new Map(collections).set(event.coll, { ready: true, docs });
};

export { applySnapshot };
