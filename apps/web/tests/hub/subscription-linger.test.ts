/* @layer renderer-app @kind test */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { created, LINGER_MS, unwantAndFlush, wantAndFlush } from '../helpers/subscription-rig';

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('C2 SubscriptionTable linger', { tags: ['critical'] }, () => {
  it('keeps a pair another tab still wants', () => {
    const { table, expired } = created();
    wantAndFlush(table);
    table.want('b', 'rotp', 'items');
    table.flush();
    unwantAndFlush(table);
    vi.advanceTimersByTime(LINGER_MS * 2);
    expect(expired).toEqual([]);
  });

  it('drops an unwanted pair after 10 s and sends its unsub', () => {
    const { table, expired } = created();
    wantAndFlush(table);
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
    wantAndFlush(table);
    unwantAndFlush(table);
    vi.advanceTimersByTime(LINGER_MS / 2);
    table.want('b', 'rotp', 'items');
    expect(table.flush()).toEqual([]);
    vi.advanceTimersByTime(LINGER_MS * 2);
    expect(expired).toEqual([]);
    expect(table.activeCollections()).toEqual(new Map([['rotp', ['items']]]));
  });

  it('starts a fresh linger when the pair is dropped again', () => {
    const { table, expired } = created();
    wantAndFlush(table);
    unwantAndFlush(table);
    wantAndFlush(table);
    unwantAndFlush(table);
    vi.advanceTimersByTime(LINGER_MS);
    expect(expired).toHaveLength(1);
  });
});

describe('C2 SubscriptionTable linger and wanting again', { tags: ['critical'] }, () => {
  it('subscribes again after the pair was dropped', () => {
    const { table } = created();
    wantAndFlush(table);
    unwantAndFlush(table);
    vi.advanceTimersByTime(LINGER_MS);
    table.want('a', 'rotp', 'items');
    expect(table.flush()).toEqual([{ op: 'sub', scope: 'rotp', colls: ['items'] }]);
  });

  it('skips the drop when the pair was wanted again before the next flush', () => {
    const { table, expired } = created();
    wantAndFlush(table);
    unwantAndFlush(table);
    table.want('a', 'rotp', 'items');
    vi.advanceTimersByTime(LINGER_MS);
    expect(expired).toEqual([]);
    expect(table.flush()).toEqual([]);
  });

  it('honours a custom linger and stops its timers on dispose', () => {
    const { table, expired } = created(50);
    wantAndFlush(table);
    wantAndFlush(table, 'wakes');
    unwantAndFlush(table);
    unwantAndFlush(table, 'wakes');
    vi.advanceTimersByTime(50);
    expect(expired).toHaveLength(2);
    wantAndFlush(table);
    unwantAndFlush(table);
    table.dispose();
    vi.advanceTimersByTime(LINGER_MS);
    expect(expired).toHaveLength(2);
  });
});
