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

  it('raises stalled and sync due from the live facts, in words', () => {
    const found = boardSignals(record('a'), { fault: null, stalls: 2, syncDue: true });
    expect(found.map((signal) => [signal.id, signal.text])).toEqual([['stalled', '2 stalled'], ['sync-due', 'Sync due']]);
  });

  it('says each count in words', () => {
    const found = boardSignals(record('a', { counts: { ...COUNTS, needsYou: 4, testsFailing: 8, blocked: 2, paused: 1 } }));
    expect(found.map((signal) => signal.text)).toEqual(['4 need you', '8 failing tests', '2 blocked', '1 paused']);
  });

  it('flags a missing master as neutral on the builds tab', () => {
    expect(boardSignals(record('a', { masterMissing: true }))).toEqual([{ id: 'master-missing', tone: 'neutral', count: 1, text: 'Master missing', label: 'master missing', tab: 'builds' }]);
  });

  it('shows only archived for an archived board', () => {
    const archived = record('a', { counts: { ...COUNTS, needsYou: 4 } }, { archivedAt: '2026-11-01T00:00:00.000Z' });
    expect(ids(boardSignals(archived, { fault: null, stalls: 3, syncDue: true }))).toEqual(['archived']);
  });
});
