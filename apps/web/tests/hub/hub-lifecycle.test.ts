/* @layer renderer-app @kind test */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { kinds, LINGER_MS, loaded, ready } from '../helpers/hub-frames';
import { createHubRig } from '../helpers/hub-rig';

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('C3 Hub status', { tags: ['critical'] }, () => {
  it('goes live per scope after that scope ready, and tells only the tabs that follow it', () => {
    const rig = createHubRig();
    rig.send('a', { type: 'want', scope: 'rotp', coll: 'items' });
    rig.send('b', { type: 'want', scope: '_portfolio', coll: 'stalls' });
    rig.status('live');
    rig.frame(ready('rotp', ['items']));
    expect(rig.to('a')).toEqual([
      { type: 'status', scope: 'rotp', status: 'connecting' },
      { type: 'status', scope: 'rotp', status: 'live' },
    ]);
    expect(rig.to('b')).toEqual([{ type: 'status', scope: '_portfolio', status: 'connecting' }]);
  });

  it('follows the connection down and up again', () => {
    const rig = loaded();
    rig.send('b', { type: 'want', scope: '_portfolio', coll: 'stalls' });
    rig.status('unavailable');
    expect(rig.to('a').at(-1)).toEqual({ type: 'status', scope: 'rotp', status: 'unavailable' });
    expect(rig.to('b').at(-1)).toEqual({ type: 'status', scope: '_portfolio', status: 'unavailable' });
    rig.status('live');
    rig.frame(ready('rotp', ['items']));
    expect(rig.to('a').at(-1)).toEqual({ type: 'status', scope: 'rotp', status: 'live' });
    expect(rig.to('b').at(-1)).toEqual({ type: 'status', scope: '_portfolio', status: 'unavailable' });
  });

  it('gives a tab that follows a scope while the connection is unauthorized that status at once', () => {
    const rig = createHubRig();
    rig.status('unauthorized');
    rig.send('a', { type: 'want', scope: 'rotp', coll: 'items' });
    expect(rig.to('a')).toEqual([{ type: 'status', scope: 'rotp', status: 'unauthorized' }]);
  });
});

describe('C3 Hub tabs', { tags: ['critical'] }, () => {
  it('unpresences at once when a tab is dropped, and when it says bye', () => {
    const rig = createHubRig();
    rig.send('a', { type: 'presence', scope: 'rotp', thread: 'th-1' });
    rig.send('b', { type: 'presence', scope: 'rotp', thread: 'th-1' });
    expect(rig.ops()).toEqual([
      { op: 'presence', scope: 'rotp', tab: 'a', thread: 'th-1' },
      { op: 'presence', scope: 'rotp', tab: 'b', thread: 'th-1' },
    ]);
    rig.hub.dropTab('a');
    rig.send('b', { type: 'bye' });
    expect(rig.ops().slice(2)).toEqual([
      { op: 'unpresence', scope: 'rotp', tab: 'a' },
      { op: 'unpresence', scope: 'rotp', tab: 'b' },
    ]);
    expect(rig.hub.tabCount()).toBe(0);
  });

  it('untails a dropped tab and unpresences an unpresence message', () => {
    const rig = createHubRig();
    rig.send('a', { type: 'tail', id: 't1', key: 'rotp', rid: 'R-1', after: 0 });
    rig.send('a', { type: 'presence', scope: 'rotp', thread: 'th-1' });
    rig.send('a', { type: 'unpresence', scope: 'rotp' });
    rig.hub.dropTab('a');
    expect(rig.ops().slice(2)).toEqual([
      { op: 'unpresence', scope: 'rotp', tab: 'a' },
      { op: 'untail', sub: 'a:t1' },
    ]);
  });

  it('keeps a wanted collection of a dropped tab through the linger for the others', () => {
    const rig = loaded();
    rig.send('b', { type: 'want', scope: 'rotp', coll: 'items' });
    rig.hub.dropTab('a');
    vi.advanceTimersByTime(LINGER_MS * 2);
    expect(rig.ops()).toHaveLength(1);
  });

});

describe('C3 Hub resume and reset', { tags: ['critical'] }, () => {
  it('resumes the whole state on hello: since for synced collections, tails after the last line, presences', () => {
    const rig = loaded();
    rig.send('a', { type: 'want', scope: 'rotp', coll: 'wakes' });
    rig.send('a', { type: 'presence', scope: 'rotp', thread: 'th-1' });
    rig.send('a', { type: 'tail', id: 't1', key: 'rotp', rid: 'R-1', after: 0 });
    rig.frame({ event: 'log', data: { sub: 'a:t1', seq: 7, stream: 'out', at: 'now', text: 'x' } });
    expect(rig.resume()).toEqual([
      { op: 'sub', scope: 'rotp', colls: ['items'], since: 10 },
      { op: 'sub', scope: 'rotp', colls: ['wakes'] },
      { op: 'presence', scope: 'rotp', tab: 'a', thread: 'th-1' },
      { op: 'tail', sub: 'a:t1', key: 'rotp', rid: 'R-1', after: 7 },
    ]);
  });

  it('resets a scope: tells the holders, clears the cache and subscribes again without since', () => {
    const rig = loaded();
    rig.send('b', { type: 'want', scope: '_portfolio', coll: 'stalls' });
    rig.frame({ event: 'reset', data: { scope: 'rotp', reason: 'resync' } });
    expect(rig.to('a').slice(-2)).toEqual([
      { type: 'reset', scope: 'rotp' },
      { type: 'status', scope: 'rotp', status: 'connecting' },
    ]);
    expect(kinds(rig.to('b'))).toEqual(['status']);
    expect(rig.ops().at(-1)).toEqual({ op: 'sub', scope: 'rotp', colls: ['items'] });
    rig.send('c', { type: 'want', scope: 'rotp', coll: 'items' });
    expect(rig.to('c').some((message) => message.type === 'snapshot')).toBe(false);
  });
});

describe('C3 Hub recovery', { tags: ['critical'] }, () => {
  it('unsubscribes before the sub after a reset so the server sends a fresh snapshot', () => {
    const rig = loaded();
    rig.frame({ event: 'reset', data: { scope: 'rotp', reason: 'resync' } });
    expect(rig.ops().slice(-2)).toEqual([
      { op: 'unsub', scope: 'rotp', colls: ['items'] },
      { op: 'sub', scope: 'rotp', colls: ['items'] },
    ]);
  });

  it('subscribes again once when a sub fails, and stops after a second failure', () => {
    const rig = createHubRig();
    rig.send('a', { type: 'want', scope: 'rotp', coll: 'items' });
    const op = { op: 'sub' as const, scope: 'rotp', colls: ['items'] };
    rig.opError(op, { code: 'busy', message: 'later' });
    expect(rig.ops()).toEqual([op, op]);
    rig.opError(op, { code: 'busy', message: 'later' });
    expect(rig.ops()).toHaveLength(2);
  });

  it('asks the server again when a tab wants a collection that is ready but holds no data', () => {
    const rig = createHubRig();
    rig.send('a', { type: 'want', scope: 'rotp', coll: 'items' });
    rig.frame(ready('rotp', ['items']));
    rig.send('b', { type: 'want', scope: 'rotp', coll: 'items' });
    expect(rig.ops().slice(-2)).toEqual([
      { op: 'unsub', scope: 'rotp', colls: ['items'] },
      { op: 'sub', scope: 'rotp', colls: ['items'] },
    ]);
  });

  it('closes the connection and stops the linger timers on dispose', () => {
    const rig = loaded();
    rig.send('a', { type: 'unwant', scope: 'rotp', coll: 'items' });
    rig.hub.dispose();
    vi.advanceTimersByTime(LINGER_MS * 2);
    expect(rig.link.close).toHaveBeenCalled();
    expect(rig.ops()).toHaveLength(1);
  });
});
