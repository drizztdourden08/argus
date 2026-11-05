/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { RunRow } from '../../src/data/runs';
import type { Card } from '../../src/data/types';
import { attentionOf, cardTone, leadState, masterLine } from '../../src/screens/overview/boards/model';
import { cardAt } from './helpers';

const NOW = Date.parse('2026-10-26T12:00:00.000Z');
const minutesAgo = (minutes: number): string => new Date(NOW - minutes * 60000).toISOString();

const card = (key: string, extra: Partial<Card> = {}): Card => cardAt(key, NOW, extra);

describe('K1 the state of a lead, from its card', { tags: ['critical'] }, () => {
  it('is not started with no board', () => {
    expect(leadState(card('a', { state: 'not started', coordinator: undefined }), NOW)).toBe('not started');
  });

  it('is paused or idle with no live coordinator, by whether threads are paused', () => {
    expect(leadState(card('a', { coordinator: null }), NOW)).toBe('idle');
    expect(leadState(card('a', { coordinator: null, counts: { threads: 0, working: 0, blocked: 0, needsYou: 0, itemsOpen: 0, done: 0, paused: 2 } }), NOW)).toBe('paused');
  });

  it('is stale when the coordinator was not seen for an hour', () => {
    expect(leadState(card('a', { coordinator: { session: 's', link: null, lastSeen: minutesAgo(61), activity: 'working', note: null } }), NOW)).toBe('stale');
    expect(leadState(card('a', { coordinator: { session: 's', link: null, lastSeen: minutesAgo(59), activity: 'idle', note: null } }), NOW)).toBe('idle');
  });

  it('is running when the coordinator works, or any thread or agent works, else idle', () => {
    const base = { threads: 0, working: 0, blocked: 0, needsYou: 0, itemsOpen: 0, done: 0 };
    expect(leadState(card('a', { coordinator: { session: 's', link: null, lastSeen: minutesAgo(1), activity: 'working', note: null } }), NOW)).toBe('running');
    expect(leadState(card('a', { counts: { ...base, working: 1 } }), NOW)).toBe('running');
    expect(leadState(card('a', { counts: { ...base, agents: 2 } }), NOW)).toBe('running');
    expect(leadState(card('a', { counts: base }), NOW)).toBe('idle');
  });
});

describe('K1 what asks for the owner', { tags: ['critical'] }, () => {
  const base = { threads: 0, working: 0, blocked: 0, needsYou: 0, itemsOpen: 0, done: 0 };

  it('counts what needs you, the failing tests, the blocked threads and the open decisions', () => {
    const hot = card('a', { counts: { ...base, needsYou: 2, testsFailing: 1, blocked: 3 }, attention: ['Q-1 needs a decision: x', 'T-2 failing: y', 'Q-4 needs a decision: z'] });
    expect(attentionOf(hot)).toEqual({ needs: 2, failing: 1, blocked: 3, decisions: 2 });
    expect(attentionOf(card('b'))).toEqual({ needs: 0, failing: 0, blocked: 0, decisions: 0 });
  });

  it('tones a card danger for failing or blocked, warning for needs you, none otherwise', () => {
    expect(cardTone({ needs: 1, failing: 1, blocked: 0, decisions: 0 })).toBe('danger');
    expect(cardTone({ needs: 0, failing: 0, blocked: 1, decisions: 0 })).toBe('danger');
    expect(cardTone({ needs: 1, failing: 0, blocked: 0, decisions: 0 })).toBe('warning');
    expect(cardTone({ needs: 0, failing: 0, blocked: 0, decisions: 0 })).toBeUndefined();
  });
});

describe('K1 the master launcher line of a card', { tags: ['critical'] }, () => {
  const master = { name: 'master', title: 'Main', cmd: 'game run', current: true, window: false, play: 'launcher-master', updatedAt: null, by: null };
  const runRow = (extra: Partial<RunRow>): RunRow => ({
    id: 'R-9', scope: 'alpha', name: 'launcher-master', kind: 'launch', status: 'running', exitCode: null, signal: null, pid: 1, visible: false, stdin: false,
    origin: 'page', by: 'owner', agent: null, reason: null, queuedAt: minutesAgo(2), startedAt: minutesAgo(1), endedAt: null, logBytes: 0, logDropped: 0,
    orphan: false, error: null, lastLine: 'window open', cmd: 'game run', cwd: 'x:/game', ...extra,
  });

  it('says no master set, and says so in red when the thread must set one', () => {
    expect(masterLine(card('a', { master: null }))).toEqual({ head: 'no master set', at: null, tail: '', tone: 'neutral', running: false });
    expect(masterLine(card('a', { master: null, masterMissing: true }))).toEqual({
      head: 'no master launcher: the thread must set one', at: null, tail: '', tone: 'danger', running: false,
    });
  });

  it('says nothing for a master that never ran', () => {
    expect(masterLine(card('a', { master }))).toEqual({ head: '', at: null, tail: '', tone: 'neutral', running: false });
  });

  it('shows a live run with its start and last line, and offers Stop', () => {
    const run = runRow({});
    expect(masterLine(card('a', { master: { ...master, lastRun: run } }))).toEqual({ head: 'running', at: run.startedAt, tail: ' · window open', tone: 'success', running: true });
  });

  it('counts a queued run from the moment it was queued', () => {
    const run = runRow({ status: 'queued', startedAt: null, lastLine: null });
    expect(masterLine(card('a', { master: { ...master, lastRun: run } }))).toMatchObject({ head: 'Queued, waiting for a free slot', at: null, tail: '', running: true });
  });

  it('shows how a finished run ended: failed with its exit code in red, other ends quiet', () => {
    const failed = runRow({ status: 'failed', exitCode: 2, endedAt: minutesAgo(4) });
    expect(masterLine(card('a', { master: { ...master, lastRun: failed } }))).toEqual({ head: 'failed (exit 2)', at: failed.endedAt, tail: ' · window open', tone: 'danger', running: false });
    const exited = runRow({ status: 'exited', exitCode: 0, endedAt: minutesAgo(3) });
    expect(masterLine(card('a', { master: { ...master, lastRun: exited } }))).toMatchObject({ head: 'exited 0', at: exited.endedAt, tone: 'info' });
  });
});
