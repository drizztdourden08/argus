/* @layer renderer-app @kind test */
import { cleanup, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { isFresh, useUnread } from '../../src/screens/overview/seen';

const KEY = 'argus.test.seen';
const NOW = Date.parse('2026-10-26T12:00:00.000Z');
const minutesAgo = (minutes: number): string => new Date(NOW - minutes * 60000).toISOString();

interface Props {
  items: { at?: string }[];
  active: boolean;
}

const mount = (props: Props) => renderHook((current: Props) => useUnread(KEY, current.items, current.active), { initialProps: props });

beforeEach(() => {
  localStorage.clear();
  vi.useFakeTimers({ toFake: ['Date'] });
  vi.setSystemTime(NOW);
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.useRealTimers();
});

describe('S1 unread since the last visit', { tags: ['standard'] }, () => {
  it('counts every entry with a time when the tab was never opened, and skips one with no time', () => {
    const view = mount({ items: [{ at: minutesAgo(5) }, { at: minutesAgo(1) }, {}], active: false });
    expect(view.result.current.unread).toHaveLength(2);
  });

  it('counts only what is newer than the last visit that was remembered', () => {
    localStorage.setItem(KEY, minutesAgo(3));
    const view = mount({ items: [{ at: minutesAgo(5) }, { at: minutesAgo(1) }], active: false });
    expect(view.result.current.unread).toHaveLength(1);
  });

  it('counts nothing while the tab is open, reads everything, and remembers the time', () => {
    const view = mount({ items: [{ at: minutesAgo(5) }], active: true });
    expect(view.result.current.unread).toEqual([]);
    expect(localStorage.getItem(KEY)).toBe(new Date(NOW).toISOString());
  });

  it('keeps what was read as read after the tab is left, and counts what arrives afterwards', () => {
    const first = { at: minutesAgo(5) };
    const view = mount({ items: [first], active: true });
    view.rerender({ items: [first], active: false });
    expect(view.result.current.unread).toEqual([]);
    const later = { at: new Date(NOW + 60000).toISOString() };
    view.rerender({ items: [first, later], active: false });
    expect(view.result.current.unread).toEqual([later]);
  });

  it('survives a reload: an older entry stays read', () => {
    const first = mount({ items: [{ at: minutesAgo(5) }], active: true });
    first.unmount();
    const again = mount({ items: [{ at: minutesAgo(5) }], active: false });
    expect(again.result.current.unread).toEqual([]);
  });

  it('marks as fresh the entries newer than the previous visit while the tab is open', () => {
    localStorage.setItem(KEY, minutesAgo(3));
    const view = mount({ items: [{ at: minutesAgo(5) }, { at: minutesAgo(1) }], active: true });
    const since = view.result.current.freshSince;
    expect(isFresh(minutesAgo(1), since)).toBe(true);
    expect(isFresh(minutesAgo(5), since)).toBe(false);
  });

  it('works when the storage throws: the tab is a convenience, not a need', () => {
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
      throw new Error('blocked');
    });
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new Error('blocked');
    });
    const view = mount({ items: [{ at: minutesAgo(1) }], active: false });
    expect(view.result.current.unread).toHaveLength(1);
    view.rerender({ items: [{ at: minutesAgo(1) }], active: true });
    expect(view.result.current.unread).toEqual([]);
  });
});
