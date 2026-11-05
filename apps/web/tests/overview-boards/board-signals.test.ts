/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { boardSignals } from '../../src/screens/overview/boards/model/board-signals';
import { COUNTS, record } from './helpers';

const ids = (signals: ReturnType<typeof boardSignals>) => signals.map((signal) => signal.id);

describe('P3 the signals of a board', { tags: ['critical'] }, () => {
  it('has none for a quiet board', () => {
    expect(boardSignals(record('a'))).toEqual([]);
  });

  it('raises needs you, failing, blocked and paused from the counts, each with its tab', () => {
    const found = boardSignals(record('a', { counts: { ...COUNTS, needsYou: 2, testsFailing: 1, blocked: 3, paused: 1 } }));
    expect(found.map((signal) => [signal.id, signal.count, signal.tone, signal.tab])).toEqual([
      ['needs-you', 2, 'warn', 'overview'],
      ['failing', 1, 'bad', 'tests'],
      ['blocked', 3, 'bad', 'work'],
      ['paused', 1, 'muted', 'work'],
    ]);
  });

  it('raises stalled, fault and sync due from the live facts', () => {
    const live = { fault: { since: 's', reason: 'board-closed' }, stalls: 2, syncDue: true };
    const found = boardSignals(record('a'), live);
    expect(ids(found)).toEqual(['stalled', 'fault', 'sync-due']);
    expect(found.find((signal) => signal.id === 'fault')?.label).toBe('board closed');
    expect(boardSignals(record('a'), { ...live, fault: { since: 's', reason: 'x' } }).find((signal) => signal.id === 'fault')?.label).toBe('wake failed');
  });

  it('flags a missing master as neutral on the builds tab', () => {
    expect(boardSignals(record('a', { masterMissing: true }))).toEqual([{ id: 'master-missing', tone: 'neutral', count: 1, label: 'master missing', tab: 'builds' }]);
  });

  it('shows a running lead as running', () => {
    expect(ids(boardSignals(record('a', { counts: { ...COUNTS, agents: 1, working: 1 } })))).toEqual(['running']);
  });

  it('shows only archived for an archived board', () => {
    const archived = record('a', { counts: { ...COUNTS, needsYou: 4 } }, { archivedAt: '2026-11-01T00:00:00.000Z' });
    expect(ids(boardSignals(archived, { fault: null, stalls: 3, syncDue: true }))).toEqual(['archived']);
  });
});
