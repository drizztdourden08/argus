/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { joinBoards } from '../../src/data/boards';
import { boardRecord } from '../../src/screens/overview/boards/model/board-record';
import { COUNTS, NOW, card, item, record } from './helpers';

describe('P3 the board record, a board doc joined to its card', { tags: ['critical'] }, () => {
  it('joins each doc to the card of the same key, sorted by key', () => {
    const joined = joinBoards([{ id: 'BD-2', key: 'b' }, { id: 'BD-1', key: 'a' }, { id: 'BD-3', key: 'z' }], [card('a'), card('b')]);
    expect(joined.map((entry) => [entry.doc.key, entry.card?.key ?? null])).toEqual([['a', 'a'], ['b', 'b'], ['z', null]]);
  });

  it('takes name and icon from the doc first, then the card, then the key', () => {
    expect(record('a', { name: 'Alpha', icon: 'star' }).name).toBe('Alpha');
    expect(record('a', { name: 'Alpha', icon: 'star' }, { name: 'Doc', icon: 'bolt' })).toMatchObject({ name: 'Doc', icon: 'bolt' });
    expect(record('a', null)).toMatchObject({ name: 'a', icon: null, palette: null });
  });

  it('carries goal, note, id, rev and the archived flag from the doc', () => {
    const found = record('a', {}, { goal: 'ship', note: 'n', rev: 4, archivedAt: '2026-11-01T00:00:00.000Z' });
    expect(found).toMatchObject({ id: 'BD-a', goal: 'ship', note: 'n', rev: 4, archived: true });
    expect(record('a')).toMatchObject({ goal: '', note: '', archived: false });
  });

  it('fills the counts with zeros and keeps done', () => {
    expect(record('a', null).counts.done).toBe(0);
    expect(record('a', { counts: { ...COUNTS, done: 5, itemsOpen: 2 } }).counts).toMatchObject({ done: 5, itemsOpen: 2, agents: 0 });
  });

  it('is not started with no card or a not started card', () => {
    expect(record('a', null)).toMatchObject({ started: false, lead: 'not started' });
    expect(record('a', { state: 'not started', lead: undefined })).toMatchObject({ started: false });
    expect(record('a').started).toBe(true);
  });

  it('knows a master and a missing one', () => {
    const master = { name: 'm', title: null, cmd: null, current: true, window: false, play: 'm', updatedAt: null, by: null };
    expect(record('a', { master }).hasMaster).toBe(true);
    expect(record('a', { masterMissing: true })).toMatchObject({ hasMaster: false, masterMissing: true });
    expect(boardRecord(item('a', { master }), NOW).masterRunning).toBe(false);
  });
});
