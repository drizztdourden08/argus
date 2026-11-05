/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { arrangeEntries } from '../../src/screens/overview/boards/behavior/arrange-entries';
import { faultOf } from '../../src/screens/overview/boards/behavior/fault-of';
import { groupEntries } from '../../src/screens/overview/boards/behavior/group-entries';
import type { BoardEntry } from '../../src/screens/overview/boards/boards-view.type';
import { COUNTS, record } from './helpers';

const QUIET = { fault: null, stalls: 0, syncDue: false };

const entry = (key: string, extra: Parameters<typeof record>[1] = {}, updatedAt = ''): BoardEntry => ({ record: record(key, extra), card: null, live: QUIET, updatedAt });

const NEEDS = { counts: { ...COUNTS, needsYou: 2 } };

describe('P6 the boards view', { tags: ['critical'] }, () => {
  it('puts the boards that need you first when sorted by attention', () => {
    const entries = [entry('beta'), entry('alpha', NEEDS)];
    const keys = arrangeEntries(entries, '', { key: 'attention', direction: 'asc' }).map((item) => item.record.key);
    expect(keys).toEqual(['alpha', 'beta']);
  });

  it('sorts by name and flips the order on descending', () => {
    const entries = [entry('beta'), entry('alpha')];
    expect(arrangeEntries(entries, '', { key: 'name', direction: 'desc' }).map((item) => item.record.key)).toEqual(['beta', 'alpha']);
  });

  it('keeps only the boards that match the search in name, key or id', () => {
    const entries = [entry('beta'), entry('alpha')];
    expect(arrangeEntries(entries, 'ALP', { key: 'name', direction: 'asc' }).map((item) => item.record.key)).toEqual(['alpha']);
  });

  it('groups by attention with the quiet boards after the ones that need you, and one group for none', () => {
    const entries = [entry('beta'), entry('alpha', NEEDS)];
    expect(groupEntries(entries, 'attention').map((group) => group.label)).toEqual(['Needs you', 'Running']);
    expect(groupEntries(entries, 'none').map((group) => group.entries.length)).toEqual([2]);
    expect(groupEntries([], 'none')).toEqual([]);
  });

  it('reads a wake fault from the registry and a closed board when no wake failed', () => {
    const rows = [{ id: 'r', project: 'beta', thread: 'lead', state: 'idle', boardRequired: true, boardOpen: false }] as never;
    expect(faultOf(rows, 'beta')?.reason).toBe('board-closed');
    expect(faultOf(rows, 'alpha')).toBeNull();
  });
});
