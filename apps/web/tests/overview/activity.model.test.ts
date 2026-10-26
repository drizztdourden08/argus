/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { ActivityItem } from '../../src/screens/overview/activity/model';
import { countLevels, kindDefs, levelOf, needsAttention, sortActivity, toLogRow } from '../../src/screens/overview/activity/model';
import { kindOf } from '../../src/screens/overview/activity/model/kind-of';

const NOW = Date.parse('2026-10-26T12:00:00.000Z');
const minutesAgo = (minutes: number): string => new Date(NOW - minutes * 60000).toISOString();

const entry = (id: string, extra: Partial<ActivityItem> = {}): ActivityItem => ({ id, at: minutesAgo(1), kind: 'note', level: 'info', text: 'text', ...extra });

const context = { now: NOW, freshSince: Number.NaN, nameOf: (key: string) => (key === 'rotp' ? 'Relic of the Past' : key) };

describe('the feed order and the level and kind of an entry', () => {
  it('runs oldest first so the newest sits at the bottom of the log, ties by the number in the id', () => {
    const rows = [entry('a-10', { at: minutesAgo(1) }), entry('a-2', { at: minutesAgo(5) }), entry('a-9', { at: minutesAgo(1) })];
    expect(sortActivity(rows).map((row) => row.id)).toEqual(['a-2', 'a-9', 'a-10']);
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

describe('the kinds of the log panel', () => {
  it('lists the known kinds, then the kinds the feed carries that are new', () => {
    const ids = kindDefs([entry('1', { kind: 'mystery' }), entry('2', { kind: 'fault' })]).map((kind) => kind.id);
    expect(ids).toEqual(['fault', 'process', 'kill', 'close', 'run', 'wake', 'error', 'note', 'mystery']);
  });

  it('gives each kind a tone, and a quiet one to a kind it does not know', () => {
    const defs = kindDefs([entry('1', { kind: 'mystery' })]);
    expect(defs.find((kind) => kind.id === 'fault')?.tone).toBe('danger');
    expect(defs.find((kind) => kind.id === 'mystery')?.tone).toBe('muted');
  });
});

describe('a row of the log', () => {
  it('shows the time ago, the kind, the text, and the project and thread in the tag', () => {
    const row = toLogRow(entry('a-1', { at: minutesAgo(5), kind: 'run', text: 'R-3 play failed', project: 'rotp', thread: 'lead' }), context);
    expect(row).toEqual({ id: 'a-1', gutter: '5m ago', tag: 'Relic of the Past / lead', kind: 'run', message: 'R-3 play failed' });
  });

  it('puts the level in the tag for a warning or an error, and nothing for info', () => {
    expect(toLogRow(entry('a', { level: 'error', project: 'rotp' }), context).tag).toBe('error Relic of the Past');
    expect(toLogRow(entry('a', { level: 'warn' }), context).tag).toBe('warn');
    expect(toLogRow(entry('a', { level: 'info' }), context).tag).toBe('');
  });

  it('marks the entries newer than the last visit as new', () => {
    const row = toLogRow(entry('a', { at: minutesAgo(1) }), { ...context, freshSince: NOW - 5 * 60000 });
    expect(row.tag).toBe('new');
    const old = toLogRow(entry('a', { at: minutesAgo(10) }), { ...context, freshSince: NOW - 5 * 60000 });
    expect(old.tag).toBe('');
  });

  it('is still a row when the doc has no time, no kind, no level and no text', () => {
    expect(toLogRow({ id: 'a-7' }, context)).toEqual({ id: 'a-7', gutter: '', tag: '', kind: 'note', message: '' });
  });
});
