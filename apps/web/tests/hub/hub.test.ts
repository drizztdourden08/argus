/* @layer renderer-app @kind test */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { change, changes, kinds, LINGER_MS, loaded, RUN, snap } from '../helpers/hub-frames';
import { createHubRig } from '../helpers/hub-rig';

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('C3 Hub subscriptions', { tags: ['critical'] }, () => {
  it('opens the connection and sends one sub for what a tab wants', () => {
    const rig = createHubRig();
    rig.send('a', { type: 'want', scope: 'rotp', coll: 'items' });
    rig.send('a', { type: 'want', scope: 'rotp', coll: 'threads' });
    expect(rig.link.open).toHaveBeenCalled();
    expect(rig.ops()).toEqual([
      { op: 'sub', scope: 'rotp', colls: ['items'] },
      { op: 'sub', scope: 'rotp', colls: ['threads'] },
    ]);
  });

  it('serves a second tab from memory when the collection is held, with no server op', () => {
    const rig = loaded();
    const before = rig.ops().length;
    rig.send('b', { type: 'want', scope: 'rotp', coll: 'items' });
    expect(rig.ops()).toHaveLength(before);
    expect(rig.to('b')).toEqual([
      { type: 'status', scope: 'rotp', status: 'live' },
      { type: 'snapshot', scope: 'rotp', seq: 10, coll: 'items', docs: [{ id: 'a', data: { n: 1 } }] },
    ]);
  });

  it('sends one sub for a new collection only', () => {
    const rig = loaded();
    rig.send('b', { type: 'want', scope: 'rotp', coll: 'items' });
    rig.send('b', { type: 'want', scope: 'rotp', coll: 'wakes' });
    expect(rig.ops().slice(1)).toEqual([{ op: 'sub', scope: 'rotp', colls: ['wakes'] }]);
  });

  it('does not send a sub twice while the snapshot is still on its way', () => {
    const rig = createHubRig();
    rig.send('a', { type: 'want', scope: 'rotp', coll: 'items' });
    rig.send('b', { type: 'want', scope: 'rotp', coll: 'items' });
    expect(rig.ops()).toHaveLength(1);
    rig.frame(snap('rotp', 'items'));
    expect(kinds(rig.to('b'))).toEqual(['status', 'snapshot']);
  });

  it('serves every wanted collection of an announce from memory and ignores another protocol version', () => {
    const rig = loaded();
    const wants = [{ scope: 'rotp', coll: 'items' }];
    rig.send('c', { type: 'announce', tabId: 'c', v: 99, wants, presences: [], tails: [] });
    expect(rig.to('c')).toEqual([]);
    expect(rig.hub.tabCount()).toBe(1);
    rig.send('c', { type: 'announce', tabId: 'c', v: 1, wants, presences: [], tails: [] });
    expect(kinds(rig.to('c'))).toEqual(['status', 'snapshot']);
    expect(rig.ops()).toHaveLength(1);
  });

  it('keeps a collection through the linger and drops it with an unsub after', () => {
    const rig = loaded();
    rig.send('a', { type: 'unwant', scope: 'rotp', coll: 'items' });
    vi.advanceTimersByTime(LINGER_MS - 1);
    rig.send('b', { type: 'want', scope: 'rotp', coll: 'items' });
    vi.advanceTimersByTime(LINGER_MS * 2);
    expect(rig.ops()).toHaveLength(1);
    rig.send('b', { type: 'unwant', scope: 'rotp', coll: 'items' });
    vi.advanceTimersByTime(LINGER_MS);
    expect(rig.ops().at(-1)).toEqual({ op: 'unsub', scope: 'rotp', colls: ['items'] });
    rig.send('c', { type: 'want', scope: 'rotp', coll: 'items' });
    expect(rig.to('c').some((message) => message.type === 'snapshot')).toBe(false);
    expect(rig.ops().at(-1)).toEqual({ op: 'sub', scope: 'rotp', colls: ['items'] });
  });
});

describe('C3 Hub routing', { tags: ['critical'] }, () => {
  it('sends a snapshot and changes only to the tabs that want that collection', () => {
    const rig = createHubRig();
    rig.send('a', { type: 'want', scope: 'rotp', coll: 'items' });
    rig.send('b', { type: 'want', scope: 'rotp', coll: 'wakes' });
    rig.frame(snap('rotp', 'items'));
    rig.frame(snap('rotp', 'wakes'));
    rig.frame(changes(11, change('items', 'x'), change('wakes', 'y')));
    const forA = rig.to('a').filter((message) => message.type !== 'status');
    const forB = rig.to('b').filter((message) => message.type !== 'status');
    expect(forA.map((message) => message.type)).toEqual(['snapshot', 'changes']);
    expect(forA[1]).toMatchObject({ seq: 11, changes: [change('items', 'x')] });
    expect(forB[1]).toMatchObject({ seq: 11, changes: [change('wakes', 'y')] });
    expect(forA[0]).toMatchObject({ coll: 'items' });
  });

  it('sends no changes message to a tab none of the changes concern', () => {
    const rig = loaded();
    rig.send('b', { type: 'want', scope: 'rotp', coll: 'wakes' });
    const before = rig.to('b').length;
    rig.frame(changes(11, change('items', 'x')));
    expect(rig.to('b')).toHaveLength(before);
  });

  it('keeps the cache current so a later tab gets the changed docs', () => {
    const rig = loaded();
    rig.frame(changes(12, change('items', 'x')));
    rig.send('b', { type: 'want', scope: 'rotp', coll: 'items' });
    expect(rig.to('b').at(-1)).toMatchObject({ type: 'snapshot', seq: 12, docs: [{ id: 'a' }, { id: 'x', data: { n: 2 } }] });
  });

  it('sends tail frames only to the tab that owns the sub, under the tab own id', () => {
    const rig = createHubRig();
    rig.send('a', { type: 'tail', id: 't1', key: 'rotp', rid: 'R-1', after: 0 });
    rig.send('b', { type: 'tail', id: 't1', key: 'rotp', rid: 'R-2', after: 0 });
    expect(rig.ops()).toEqual([
      { op: 'tail', sub: 'a:t1', key: 'rotp', rid: 'R-1', after: 0 },
      { op: 'tail', sub: 'b:t1', key: 'rotp', rid: 'R-2', after: 0 },
    ]);
    rig.frame({ event: 'log', data: { sub: 'b:t1', seq: 4, stream: 'out', at: 'now', text: 'hi' } });
    rig.frame({ event: 'run', data: { sub: 'b:t1', run: RUN } });
    rig.frame({ event: 'tail-end', data: { sub: 'b:t1' } });
    expect(rig.to('a')).toEqual([]);
    expect(rig.to('b')).toEqual([
      { type: 'log', id: 't1', seq: 4, stream: 'out', at: 'now', text: 'hi' },
      { type: 'run', id: 't1', run: RUN },
      { type: 'tail-end', id: 't1' },
    ]);
  });

  it('drops frames of a sub no tab owns and ends a tail the server refused', () => {
    const rig = createHubRig();
    rig.frame({ event: 'log', data: { sub: 'ghost:1', seq: 1, stream: 'out', at: 'now', text: 'x' } });
    rig.send('a', { type: 'tail', id: 't1', key: 'rotp', rid: 'R-1', after: 0 });
    rig.opError({ op: 'tail', sub: 'a:t1', key: 'rotp', rid: 'R-1', after: 0 }, { code: 'gone', message: 'no run' });
    expect(rig.to('a')).toEqual([{ type: 'tail-end', id: 't1' }]);
  });
});
