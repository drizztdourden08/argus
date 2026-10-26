/* @layer renderer-app @kind test */
import type { FakeSource } from './fake-source';
import { snapshot } from './snapshot';

const BOARD_COLLECTIONS = ['actions', 'agents', 'builds', 'entries', 'items', 'log', 'meta', 'registry', 'tests', 'threads'];

type Seed = Partial<Record<string, Record<string, object>>>;

const clock = { seq: 100 };

const seedBoard = (source: FakeSource | undefined, docs: Seed = {}, scope = 'rotp'): void => {
  for (const coll of BOARD_COLLECTIONS) source?.emit('snapshot', snapshot(scope, coll, docs[coll] ?? {}));
  source?.emit('ready', { seq: 10 });
};

const changeBoard = (source: FakeSource | undefined, coll: string, id: string, data: object | null): void => {
  clock.seq += 1;
  const change = data ? { coll, id, op: 'set', data } : { coll, id, op: 'delete' };
  source?.emit('changes', { scope: 'rotp', seq: clock.seq, changes: [change] });
};

export { changeBoard, seedBoard };
