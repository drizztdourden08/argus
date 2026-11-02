/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { depBadge, filterDeps, sortDeps, waitMs } from '../../src/screens/overview/deps/model';
import { dep } from './helpers/deps';

const MIN = 60_000;
const DAY = 86_400_000;
const NOW = Date.now();
const before = (ms: number): string => new Date(NOW - ms).toISOString();

const open = (id: string, waited: number) => dep(id, { state: 'open', openedAt: before(waited) });

describe('K1 the filters of the Dependencies tab', { tags: ['critical'] }, () => {
  const docs = [open('D-1', MIN), dep('D-2', { state: 'opening' }), dep('D-3', { state: 'blocked' }), dep('D-4', { state: 'done' }), dep('D-5', { state: 'gone' }), dep('D-6', { state: 'cancelled' })];

  it('keeps the opening and open requests on Open', () => {
    expect(filterDeps(docs, 'open').map((doc) => doc.id)).toEqual(['D-1', 'D-2']);
  });

  it('keeps only the blocked ones on Blocked', () => {
    expect(filterDeps(docs, 'blocked').map((doc) => doc.id)).toEqual(['D-3']);
  });

  it('keeps done, dropped, gone and cancelled on Resolved', () => {
    expect(filterDeps(docs, 'resolved').map((doc) => doc.id)).toEqual(['D-4', 'D-5', 'D-6']);
  });

  it('keeps every request on All', () => {
    expect(filterDeps(docs, 'all')).toHaveLength(6);
  });
});

describe('K1 the order and the badge of the Dependencies tab', { tags: ['critical'] }, () => {
  it('puts the longest wait first and the resolved requests after the live ones', () => {
    const docs = [open('D-1', MIN), dep('D-2', { state: 'done', openedAt: before(9 * DAY), resolvedAt: before(DAY) }), open('D-3', 2 * DAY), dep('D-4', { state: 'blocked', openedAt: before(5 * MIN) })];
    expect(sortDeps(docs, NOW).map((doc) => doc.id)).toEqual(['D-3', 'D-4', 'D-1', 'D-2']);
  });

  it('measures a resolved wait up to its resolution, not up to now', () => {
    expect(waitMs(dep('D-1', { state: 'done', openedAt: before(3 * DAY), resolvedAt: before(2 * DAY) }), NOW)).toBe(DAY);
  });

  it('counts the blocked requests and the open ones over 3 days, never a fresh open one or a resolved one', () => {
    const docs = [open('D-1', MIN), open('D-2', 4 * DAY), dep('D-3', { state: 'blocked', openedAt: before(MIN) }), dep('D-4', { state: 'done', openedAt: before(9 * DAY), resolvedAt: before(DAY) })];
    expect(depBadge(docs, NOW)).toBe(2);
  });
});
