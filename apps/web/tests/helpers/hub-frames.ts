/* @layer renderer-app @kind test */
import type { MuxFrame } from '../../src/data/hub';
import type { RunRow } from '../../src/data/runs';
import type { WireChange } from '../../src/data/stream/stream.type';
import { createHubRig } from './hub-rig';
import { snapshot } from './snapshot';

const LINGER_MS = 10_000;

const RUN: RunRow = {
  id: 'R-1', scope: 'rotp', name: 'build', kind: 'launch', status: 'running', exitCode: null, signal: null, pid: 1, visible: false,
  stdin: false, origin: 'page', by: 'owner', agent: null, reason: null, queuedAt: '2026-11-03T10:00:00Z', startedAt: null, endedAt: null,
  logBytes: 0, logDropped: 0, orphan: false, error: null, lastLine: null, cmd: 'x', cwd: 'x:/y',
};

const snap = (scope: string, coll: string, seq = 10): MuxFrame => ({ event: 'snapshot', data: snapshot(scope, coll, { a: { n: 1 } }, seq) });
const ready = (scope: string, colls: string[], seq = 10): MuxFrame => ({ event: 'ready', data: { scope, seq, colls } });
const change = (coll: string, id: string): WireChange => ({ coll, id, op: 'set', data: { n: 2 } });
const changes = (seq: number, ...list: WireChange[]): MuxFrame => ({ event: 'changes', data: { scope: 'rotp', seq, changes: list } });
const kinds = (messages: { type: string }[]): string[] => messages.map((message) => message.type);

const loaded = () => {
  const rig = createHubRig();
  rig.send('a', { type: 'want', scope: 'rotp', coll: 'items' });
  rig.frame(snap('rotp', 'items'));
  rig.frame(ready('rotp', ['items']));
  return rig;
};

export { change, changes, kinds, LINGER_MS, loaded, ready, RUN, snap };
