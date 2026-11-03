/* @layer renderer-app @kind test */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { MuxOp } from '../../src/data/hub/hub.type';
import { SubscriptionTable } from '../../src/data/hub/subscription-table';

const LINGER_MS = 10_000;

const created = (lingerMs?: number) => {
  const expired: MuxOp[][] = [];
  const table = new SubscriptionTable({ onExpire: (ops) => expired.push(ops), lingerMs });
  return { table, expired };
};

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('C2 SubscriptionTable union and diff', { tags: ['critical'] }, () => {
  it('sends one sub for the union of what two tabs want', () => {
    const { table } = created();
    table.want('a', 'rotp', 'items');
    table.want('b', 'rotp', 'items');
    table.want('b', 'rotp', 'threads');
    expect(table.flush()).toEqual([{ op: 'sub', scope: 'rotp', colls: ['items', 'threads'] }]);
  });

  it('sends only the collection that is new on a later flush', () => {
    const { table } = created();
    table.want('a', 'rotp', 'items');
    table.flush();
    table.want('b', 'rotp', 'items');
    table.want('b', 'rotp', 'wakes');
    expect(table.flush()).toEqual([{ op: 'sub', scope: 'rotp', colls: ['wakes'] }]);
    expect(table.flush()).toEqual([]);
  });

  it('keeps one sub per scope', () => {
    const { table } = created();
    table.want('a', 'rotp', 'items');
    table.want('a', '_portfolio', 'stalls');
    expect(table.flush()).toEqual([
      { op: 'sub', scope: 'rotp', colls: ['items'] },
      { op: 'sub', scope: '_portfolio', colls: ['stalls'] },
    ]);
  });

  it('lists the active collections per scope', () => {
    const { table } = created();
    table.want('a', 'rotp', 'items');
    table.want('a', 'rotp', 'wakes');
    table.flush();
    expect(table.activeCollections()).toEqual(new Map([['rotp', ['items', 'wakes']]]));
  });

  it('names the tabs that want a pair and the tabs that follow a scope', () => {
    const { table } = created();
    table.want('a', 'rotp', 'items');
    table.want('b', 'rotp', 'wakes');
    table.want('c', '_portfolio', 'stalls');
    expect(table.wantingTabs('rotp', 'items')).toEqual(['a']);
    expect(table.followingTabs('rotp')).toEqual(['a', 'b']);
    expect(table.followingTabs('none')).toEqual([]);
  });

  it('replaces everything a tab held when it announces again', () => {
    const { table } = created();
    table.want('a', 'rotp', 'items');
    table.presence('a', 'rotp', 'T-1');
    table.flush();
    table.announce('a', { wants: [{ scope: 'rotp', coll: 'wakes' }], presences: [], tails: [] });
    expect(table.wantingTabs('rotp', 'items')).toEqual([]);
    expect(table.flush()).toEqual([
      { op: 'sub', scope: 'rotp', colls: ['wakes'] },
      { op: 'unpresence', scope: 'rotp', tab: 'a' },
    ]);
  });
});

describe('C2 SubscriptionTable linger', { tags: ['critical'] }, () => {
  it('keeps a pair another tab still wants', () => {
    const { table, expired } = created();
    table.want('a', 'rotp', 'items');
    table.want('b', 'rotp', 'items');
    table.flush();
    table.unwant('a', 'rotp', 'items');
    table.flush();
    vi.advanceTimersByTime(LINGER_MS * 2);
    expect(expired).toEqual([]);
  });

  it('drops an unwanted pair after 10 s and sends its unsub', () => {
    const { table, expired } = created();
    table.want('a', 'rotp', 'items');
    table.flush();
    table.unwant('a', 'rotp', 'items');
    expect(table.flush()).toEqual([]);
    vi.advanceTimersByTime(LINGER_MS - 1);
    expect(expired).toEqual([]);
    vi.advanceTimersByTime(1);
    expect(expired).toEqual([[{ op: 'unsub', scope: 'rotp', colls: ['items'] }]]);
    expect(table.activeCollections().size).toBe(0);
  });

  it('cancels the drop when a tab wants the pair again inside the linger', () => {
    const { table, expired } = created();
    table.want('a', 'rotp', 'items');
    table.flush();
    table.unwant('a', 'rotp', 'items');
    table.flush();
    vi.advanceTimersByTime(LINGER_MS / 2);
    table.want('b', 'rotp', 'items');
    expect(table.flush()).toEqual([]);
    vi.advanceTimersByTime(LINGER_MS * 2);
    expect(expired).toEqual([]);
    expect(table.activeCollections()).toEqual(new Map([['rotp', ['items']]]));
  });

  it('starts a fresh linger when the pair is dropped again', () => {
    const { table, expired } = created();
    table.want('a', 'rotp', 'items');
    table.flush();
    table.unwant('a', 'rotp', 'items');
    table.flush();
    table.want('a', 'rotp', 'items');
    table.flush();
    table.unwant('a', 'rotp', 'items');
    table.flush();
    vi.advanceTimersByTime(LINGER_MS);
    expect(expired).toHaveLength(1);
  });

  it('subscribes again after the pair was dropped', () => {
    const { table } = created();
    table.want('a', 'rotp', 'items');
    table.flush();
    table.unwant('a', 'rotp', 'items');
    table.flush();
    vi.advanceTimersByTime(LINGER_MS);
    table.want('a', 'rotp', 'items');
    expect(table.flush()).toEqual([{ op: 'sub', scope: 'rotp', colls: ['items'] }]);
  });

  it('skips the drop when the pair was wanted again before the next flush', () => {
    const { table, expired } = created();
    table.want('a', 'rotp', 'items');
    table.flush();
    table.unwant('a', 'rotp', 'items');
    table.flush();
    table.want('a', 'rotp', 'items');
    vi.advanceTimersByTime(LINGER_MS);
    expect(expired).toEqual([]);
    expect(table.flush()).toEqual([]);
  });

  it('honours a custom linger and stops its timers on dispose', () => {
    const { table, expired } = created(50);
    table.want('a', 'rotp', 'items');
    table.want('a', 'rotp', 'wakes');
    table.flush();
    table.unwant('a', 'rotp', 'items');
    table.unwant('a', 'rotp', 'wakes');
    table.flush();
    vi.advanceTimersByTime(50);
    expect(expired).toHaveLength(2);
    table.want('a', 'rotp', 'items');
    table.flush();
    table.unwant('a', 'rotp', 'items');
    table.flush();
    table.dispose();
    vi.advanceTimersByTime(LINGER_MS);
    expect(expired).toHaveLength(2);
  });
});

describe('C2 SubscriptionTable dropped tab', { tags: ['critical'] }, () => {
  it('loses its presences and tails at once and its wants after the linger', () => {
    const { table, expired } = created();
    table.want('a', 'rotp', 'items');
    table.presence('a', 'rotp', 'T-1');
    table.tail('a', '1', { key: 'rotp', rid: 'R-1', after: 0 });
    table.flush();
    table.dropTab('a');
    expect(table.flush()).toEqual([
      { op: 'unpresence', scope: 'rotp', tab: 'a' },
      { op: 'untail', sub: 'a:1' },
    ]);
    expect(table.tailOwner('a:1')).toBeUndefined();
    expect(expired).toEqual([]);
    vi.advanceTimersByTime(LINGER_MS);
    expect(expired).toEqual([[{ op: 'unsub', scope: 'rotp', colls: ['items'] }]]);
  });

  it('leaves the other tabs untouched', () => {
    const { table } = created();
    table.presence('a', 'rotp', 'T-1');
    table.presence('b', 'rotp', 'T-2');
    table.flush();
    table.dropTab('a');
    expect(table.flush()).toEqual([{ op: 'unpresence', scope: 'rotp', tab: 'a' }]);
    expect(table.resumeOps()).toEqual([{ op: 'presence', scope: 'rotp', tab: 'b', thread: 'T-2' }]);
  });
});

describe('C2 SubscriptionTable presences and tails', { tags: ['critical'] }, () => {
  it('sends a presence per tab and scope, again when the thread changes, and an unpresence on release', () => {
    const { table } = created();
    table.presence('a', 'rotp', 'T-1');
    table.presence('b', 'rotp', 'T-1');
    expect(table.flush()).toEqual([
      { op: 'presence', scope: 'rotp', thread: 'T-1', tab: 'a' },
      { op: 'presence', scope: 'rotp', thread: 'T-1', tab: 'b' },
    ]);
    table.presence('a', 'rotp', 'T-2');
    expect(table.flush()).toEqual([{ op: 'presence', scope: 'rotp', thread: 'T-2', tab: 'a' }]);
    table.unpresence('b', 'rotp');
    expect(table.flush()).toEqual([{ op: 'unpresence', scope: 'rotp', tab: 'b' }]);
    expect(table.flush()).toEqual([]);
  });

  it('starts a tail under a hub unique sub and ends it on untail', () => {
    const { table } = created();
    table.tail('a', '1', { key: 'rotp', rid: 'R-1', after: 4 });
    table.tail('b', '1', { key: 'rotp', rid: 'R-2', after: 0 });
    expect(table.flush()).toEqual([
      { op: 'tail', sub: 'a:1', key: 'rotp', rid: 'R-1', after: 4 },
      { op: 'tail', sub: 'b:1', key: 'rotp', rid: 'R-2', after: 0 },
    ]);
    expect(table.tailOwner('b:1')).toEqual({ tabId: 'b', id: '1' });
    table.untail('a', '1');
    expect(table.flush()).toEqual([{ op: 'untail', sub: 'a:1' }]);
    expect(table.tailOwner('a:1')).toBeUndefined();
  });

  it('resumes presences and tails with the tail progress', () => {
    const { table } = created();
    table.presence('a', 'rotp', 'T-1');
    table.tail('a', '1', { key: 'rotp', rid: 'R-1', after: 0 });
    table.flush();
    table.tailProgress('a:1', 7);
    table.tailProgress('a:1', 3);
    expect(table.resumeOps()).toEqual([
      { op: 'presence', scope: 'rotp', thread: 'T-1', tab: 'a' },
      { op: 'tail', sub: 'a:1', key: 'rotp', rid: 'R-1', after: 7 },
    ]);
  });

  it('does not send held presences again after an announce with the same state', () => {
    const { table } = created();
    table.presence('a', 'rotp', 'T-1');
    table.flush();
    table.announce('a', { wants: [], presences: [{ scope: 'rotp', thread: 'T-1' }], tails: [] });
    expect(table.flush()).toEqual([]);
  });
});
