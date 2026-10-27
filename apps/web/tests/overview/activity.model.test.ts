/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { ActivityItem } from '../../src/screens/overview/activity/model';
import { countLevels, filterActivity, kindCounts, kindIcon, levelOf, needsAttention, sortActivity } from '../../src/screens/overview/activity/model';
import { kindOf } from '../../src/screens/overview/activity/model/kind-of';

const NOW = Date.parse('2026-10-26T12:00:00.000Z');
const minutesAgo = (minutes: number): string => new Date(NOW - minutes * 60000).toISOString();

const entry = (id: string, extra: Partial<ActivityItem> = {}): ActivityItem => ({ id, at: minutesAgo(1), kind: 'note', level: 'info', text: 'text', ...extra });

describe('the feed order and the level and kind of an entry', () => {
  it('runs newest first, ties by the number in the id, newest number first', () => {
    const rows = [entry('a-10', { at: minutesAgo(1) }), entry('a-2', { at: minutesAgo(5) }), entry('a-9', { at: minutesAgo(1) })];
    expect(sortActivity(rows).map((row) => row.id)).toEqual(['a-10', 'a-9', 'a-2']);
  });

  it('reads a level it does not know, or none, as info, and a missing kind as note', () => {
    expect(levelOf(entry('a', { level: 'error' }))).toBe('error');
    expect(levelOf(entry('a', { level: 'shouting' }))).toBe('info');
    expect(levelOf({ id: 'a' })).toBe('info');
    expect(kindOf({ id: 'a' })).toBe('note');
    expect(kindOf(entry('a', { kind: 'fault' }))).toBe('fault');
  });

  it('needs attention when it is a warning or an error, never for info', () => {
    expect(needsAttention(entry('a', { level: 'warn' }))).toBe(true);
    expect(needsAttention(entry('a', { level: 'error' }))).toBe(true);
    expect(needsAttention(entry('a', { level: 'info' }))).toBe(false);
  });

  it('counts the levels, with all on top', () => {
    const rows = [entry('1'), entry('2', { level: 'warn' }), entry('3', { level: 'warn' }), entry('4', { level: 'error' })];
    expect(countLevels(rows)).toEqual({ all: 4, info: 1, warn: 2, error: 1 });
  });
});

describe('the filters', () => {
  const rows = [
    entry('1', { kind: 'run', level: 'error' }),
    entry('2', { kind: 'note', level: 'info' }),
    entry('3', { kind: 'run', level: 'warn' }),
  ];

  it('narrows by level and by kind, and both combine; all and an empty kind clear', () => {
    expect(filterActivity(rows, { level: 'all', kind: '' })).toHaveLength(3);
    expect(filterActivity(rows, { level: 'error', kind: '' }).map((row) => row.id)).toEqual(['1']);
    expect(filterActivity(rows, { level: 'all', kind: 'run' }).map((row) => row.id)).toEqual(['1', '3']);
    expect(filterActivity(rows, { level: 'warn', kind: 'run' }).map((row) => row.id)).toEqual(['3']);
    expect(filterActivity(rows, { level: 'warn', kind: 'note' })).toEqual([]);
  });

  it('lists the known kinds with their counts, then the kinds the feed carries that are new', () => {
    const counts = kindCounts([entry('1', { kind: 'mystery' }), entry('2', { kind: 'fault' }), entry('3', { kind: 'fault' })]);
    expect(counts.map(([kind]) => kind)).toEqual(['fault', 'process', 'kill', 'close', 'run', 'wake', 'error', 'note', 'mystery']);
    expect(counts[0]).toEqual(['fault', 2]);
    expect(counts[8]).toEqual(['mystery', 1]);
  });

  it('gives each kind an icon, and the note icon to a kind it does not know', () => {
    expect(kindIcon('fault')).toBe('triangle-alert');
    expect(kindIcon('mystery')).toBe('file-text');
  });
});
