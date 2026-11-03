/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { ScopeCache } from '../../src/data/hub/scope-cache';
import type { WireChange } from '../../src/data/stream/stream.type';
import { snapshot } from '../helpers/snapshot';

const change = (seq: number, ...changes: WireChange[]) => ({ scope: 'rotp', seq, changes });
const set = (coll: string, id: string, data: object): WireChange => ({ coll, id, op: 'set', data });

const synced = (colls: Record<string, number>) => {
  const cache = new ScopeCache();
  for (const [coll, seq] of Object.entries(colls)) cache.applySnapshot(snapshot('rotp', coll, { a: { n: 1 } }, seq));
  cache.markReady('rotp', Math.max(...Object.values(colls)), Object.keys(colls));
  return cache;
};

describe('C2 ScopeCache collections', { tags: ['critical'] }, () => {
  it('builds a collection from a snapshot and keeps the scope seq', () => {
    const cache = new ScopeCache();
    cache.applySnapshot(snapshot('rotp', 'items', { a: { n: 1 }, b: { n: 2 } }, 12));
    expect(cache.holds('rotp', 'items')).toBe(true);
    expect([...cache.collection('rotp', 'items').docs.keys()]).toEqual(['a', 'b']);
    expect(cache.seq('rotp')).toBe(12);
    expect(cache.holds('rotp', 'wakes')).toBe(false);
    expect(cache.collection('rotp', 'wakes').ready).toBe(false);
  });

  it('applies changes and reports the touched ready collections', () => {
    const cache = synced({ items: 10, wakes: 10 });
    const touched = cache.applyChanges(change(14, set('items', 'a', { n: 5 }), set('items', 'b', { n: 6 })));
    expect(touched).toEqual(['items']);
    expect(cache.collection('rotp', 'items').docs.get('a')).toEqual({ n: 5 });
    expect(cache.seq('rotp')).toBe(14);
  });

  it('removes a deleted doc', () => {
    const cache = synced({ items: 10 });
    cache.applyChanges(change(11, { coll: 'items', id: 'a', op: 'delete' }));
    expect(cache.collection('rotp', 'items').docs.has('a')).toBe(false);
  });

  it('ignores changes for a scope or a collection it does not hold', () => {
    const cache = synced({ items: 10 });
    expect(cache.applyChanges(change(12, set('ghost', 'x', {})))).toEqual([]);
    expect(cache.holds('rotp', 'ghost')).toBe(false);
    expect(new ScopeCache().applyChanges(change(12, set('items', 'x', {})))).toEqual([]);
  });

  it('keeps scopes apart', () => {
    const cache = synced({ items: 10 });
    cache.applySnapshot(snapshot('_portfolio', 'stalls', { s: {} }, 30));
    expect(cache.seq('rotp')).toBe(10);
    expect(cache.seq('_portfolio')).toBe(30);
    expect(cache.holds('rotp', 'stalls')).toBe(false);
  });

  it('drops collections and their sync state', () => {
    const cache = synced({ items: 10, wakes: 10 });
    cache.drop('rotp', ['items']);
    expect(cache.holds('rotp', 'items')).toBe(false);
    expect(cache.isSynced('rotp', 'items')).toBe(false);
    expect(cache.holds('rotp', 'wakes')).toBe(true);
    expect(() => new ScopeCache().drop('none', ['items'])).not.toThrow();
  });
});

describe('C2 ScopeCache synced state', { tags: ['critical'] }, () => {
  it('is synced only after the ready that names the collection', () => {
    const cache = new ScopeCache();
    cache.applySnapshot(snapshot('rotp', 'items', { a: {} }, 10));
    cache.applySnapshot(snapshot('rotp', 'wakes', { a: {} }, 10));
    expect(cache.isSynced('rotp', 'items')).toBe(false);
    cache.markReady('rotp', 10, ['items']);
    expect(cache.isSynced('rotp', 'items')).toBe(true);
    expect(cache.isSynced('rotp', 'wakes')).toBe(false);
  });

  it('ignores a ready for a collection it holds no snapshot of', () => {
    const cache = new ScopeCache();
    cache.applySnapshot(snapshot('rotp', 'items', {}, 10));
    cache.markReady('rotp', 10, ['items', 'ghost']);
    expect(cache.isSynced('rotp', 'ghost')).toBe(false);
    expect(() => new ScopeCache().markReady('none', 1, ['items'])).not.toThrow();
  });

  it('is not synced again once a new snapshot replaces it before the ready', () => {
    const cache = synced({ items: 10 });
    cache.applySnapshot(snapshot('rotp', 'items', { a: { n: 9 } }, 40));
    expect(cache.isSynced('rotp', 'items')).toBe(false);
    cache.markReady('rotp', 40, ['items']);
    expect(cache.isSynced('rotp', 'items')).toBe(true);
  });
});

describe('C2 ScopeCache resume ops', { tags: ['critical'] }, () => {
  it('resumes synced collections with since and the rest without', () => {
    const cache = synced({ items: 10 });
    cache.applySnapshot(snapshot('rotp', 'wakes', { a: {} }, 12));
    const wanted = new Map([['rotp', ['items', 'wakes', 'threads']]]);
    expect(cache.resumeOps(wanted)).toEqual([
      { op: 'sub', scope: 'rotp', colls: ['items'], since: 10 },
      { op: 'sub', scope: 'rotp', colls: ['wakes', 'threads'] },
    ]);
  });

  it('resumes from the oldest synced collection', () => {
    const cache = synced({ items: 10 });
    cache.applySnapshot(snapshot('rotp', 'wakes', {}, 25));
    cache.markReady('rotp', 25, ['wakes']);
    expect(cache.resumeOps(new Map([['rotp', ['items', 'wakes']]]))).toEqual([
      { op: 'sub', scope: 'rotp', colls: ['items', 'wakes'], since: 10 },
    ]);
  });

  it('moves the since of synced collections forward with the changes it applied', () => {
    const cache = synced({ items: 10, wakes: 10 });
    cache.applyChanges(change(31, set('items', 'a', { n: 2 })));
    expect(cache.resumeOps(new Map([['rotp', ['items', 'wakes']]]))).toEqual([
      { op: 'sub', scope: 'rotp', colls: ['items', 'wakes'], since: 31 },
    ]);
  });

  it('sends a plain sub for a scope it never held and nothing for an empty want', () => {
    const cache = new ScopeCache();
    const wanted = new Map([
      ['rotp', ['items']],
      ['empty', []],
    ]);
    expect(cache.resumeOps(wanted)).toEqual([{ op: 'sub', scope: 'rotp', colls: ['items'] }]);
  });

  it('never resumes a collection synced at seq 0', () => {
    const cache = synced({ items: 0 });
    expect(cache.resumeOps(new Map([['rotp', ['items']]]))).toEqual([{ op: 'sub', scope: 'rotp', colls: ['items'] }]);
  });

  it('keeps one op set per scope', () => {
    const cache = synced({ items: 10 });
    cache.applySnapshot(snapshot('_portfolio', 'stalls', {}, 20));
    cache.markReady('_portfolio', 20, ['stalls']);
    expect(
      cache.resumeOps(
        new Map([
          ['rotp', ['items']],
          ['_portfolio', ['stalls']],
        ]),
      ),
    ).toEqual([
      { op: 'sub', scope: 'rotp', colls: ['items'], since: 10 },
      { op: 'sub', scope: '_portfolio', colls: ['stalls'], since: 20 },
    ]);
  });
});

describe('C2 ScopeCache reset', { tags: ['critical'] }, () => {
  it('clears the scope, names what it held and leaves other scopes alone', () => {
    const cache = synced({ items: 10, wakes: 10 });
    cache.applySnapshot(snapshot('_portfolio', 'stalls', {}, 20));
    expect(cache.reset('rotp')).toEqual(['items', 'wakes']);
    expect(cache.holds('rotp', 'items')).toBe(false);
    expect(cache.isSynced('rotp', 'items')).toBe(false);
    expect(cache.seq('rotp')).toBe(0);
    expect(cache.holds('_portfolio', 'stalls')).toBe(true);
    expect(cache.reset('rotp')).toEqual([]);
  });

  it('resubscribes everything without since after a reset', () => {
    const cache = synced({ items: 10, wakes: 10 });
    cache.reset('rotp');
    expect(cache.resumeOps(new Map([['rotp', ['items', 'wakes']]]))).toEqual([{ op: 'sub', scope: 'rotp', colls: ['items', 'wakes'] }]);
  });
});
