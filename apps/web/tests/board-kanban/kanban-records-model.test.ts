/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { addPreset, focusChoices, pausedThreads, rangeOf } from '../../src/screens/board/tabs/kanban/model';
import { itemRecord } from '../../src/screens/board/tabs/kanban/record';
import { purgeCounts, purgeCriteria } from '../../src/screens/board/tabs/kanban/trash';
import type { PurgeData, PurgeDraft } from '../../src/screens/board/tabs/kanban/trash';
import { DAY, NOW, before, entryRef, threadRef } from './kanban-docs';

const draft = (extra: Partial<PurgeDraft>): PurgeDraft => ({ scope: 'thread', thread: '', keep: [], status: [], idle: 30, confirm: '', ...extra });

describe('K1 the presets of + Add', { tags: ['critical'] }, () => {
  it('follows the grouping of the column', () => {
    expect(addPreset('status', 'doing', '')).toEqual({ status: 'doing' });
    expect(addPreset('thread', 'idle', 'other')).toEqual({ thread: 'idle' });
    expect(addPreset('parent', 'I-5', 'other')).toEqual({ parent: 'I-5', thread: 'other' });
    expect(addPreset('parent', 'no-parent', '')).toEqual({});
    expect(addPreset('assignee', 'owner', '')).toEqual({ assignee: 'owner' });
    expect(addPreset('assignee', 'worker', '')).toEqual({ thread: 'worker' });
    expect(addPreset('status', null, 'other')).toEqual({ thread: 'other' });
  });
});

describe('K1 the range of a shift-click', { tags: ['critical'] }, () => {
  it('runs from the anchor to the click in either direction, and is the click alone without an anchor', () => {
    const order = ['a', 'b', 'c', 'd'];
    expect(rangeOf(order, 'b', 'd')).toEqual(['b', 'c', 'd']);
    expect(rangeOf(order, 'd', 'b')).toEqual(['b', 'c', 'd']);
    expect(rangeOf(order, null, 'c')).toEqual(['c']);
    expect(rangeOf(order, 'gone', 'c')).toEqual(['c']);
  });
});

describe('K1 paused threads and the focus choices', { tags: ['critical'] }, () => {
  const threads = [threadRef('a'), threadRef('b', { status: 'paused' }), threadRef('lead', { role: 'lead' }), threadRef('old', { status: 'done' })];

  it('reads the board status and the pause the owner posted that is not applied yet', () => {
    const entries = [entryRef('O-1', { kind: 'pause', author: 'human', replyTo: 'a', status: 'new' }), entryRef('O-2', { kind: 'resume', author: 'human', replyTo: 'b', status: 'new', createdAt: before(0) })];
    expect([...pausedThreads(threads, entries)].sort()).toEqual(['a']);
    expect([...pausedThreads(threads, [])]).toEqual(['b']);
  });

  it('offers the live threads without the lead and says which are paused', () => {
    const choices = focusChoices(threads, new Set(['b']));
    expect(choices.options.map((option) => option.label)).toEqual(['a', 'b (paused)']);
    expect(choices.pausedCount).toBe(1);
  });
});

describe('K1 the record of an item', { tags: ['critical'] }, () => {
  it('splits questions and decisions, finds the open ones and keeps the replies under them', () => {
    const entries = [
      entryRef('Q-1', { text: 'one?' }),
      entryRef('Q-2', { text: 'two?', createdAt: before(0) }),
      entryRef('O-1', { kind: 'followup', author: 'human', replyTo: 'Q-1', status: 'new', text: 'yes', item: 'I-1', createdAt: before(0) }),
      entryRef('D-1', { kind: 'decision', status: 'pending', text: 'go' }),
      entryRef('Q-9', { item: 'I-2', text: 'elsewhere' }),
    ];
    const record = itemRecord({ itemId: 'I-1', entries: entries as never, actions: [], log: [] });
    expect(record.questions.map((row) => [row.entry.id, row.open])).toEqual([['Q-1', false], ['Q-2', true]]);
    expect(record.questions[0]?.replies.map((reply) => reply.id)).toEqual(['O-1']);
    expect(record.decisions.map((row) => row.entry.id)).toEqual(['D-1']);
  });
});

describe('K3 what a purge removes', { tags: ['critical'] }, () => {
  const data: PurgeData = {
    threads: [threadRef('a', { lastSeen: before(40 * DAY) }), threadRef('b', { lastSeen: before(DAY) }), threadRef('lead', { role: 'lead', lastSeen: before(90 * DAY) })],
    items: [{ id: 'I-1', thread: 'a', ac: [{}, {}] }, { id: 'I-2', thread: 'b' }],
    tests: [{ id: 'T-1', item: 'I-1', thread: 'a' }],
    actions: [{ id: 'H-1', item: 'I-1', thread: 'a' }],
    entries: [{ id: 'Q-1', thread: 'a', item: 'I-1' }, { id: 'F-1', replyTo: 'Q-1' }, { id: 'Q-2', thread: 'b' }],
    log: [{ subject: 'work', entries: [{ seq: 1, ts: before(0), thread: 'a', text: 'x' }, { seq: 2, ts: before(0), thread: 'b', item: 'I-2', text: 'y' }] }],
  };

  it('follows the records of a thread, the replies to them and its log lines', () => {
    expect(purgeCounts({ threads: ['a'] }, data, NOW)).toEqual({ threads: 1, items: 1, ac: 2, tests: 1, entries: 2, actions: 1, logLines: 1 });
  });

  it('keeps the checked threads, picks idle ones, and never takes the lead', () => {
    expect(purgeCounts({ keep: ['a'] }, data, NOW).threads).toBe(1);
    expect(purgeCounts({ idleDays: 30 }, data, NOW).threads).toBe(1);
    expect(purgeCounts({ status: ['done'] }, data, NOW).threads).toBe(0);
  });

  it('writes the criteria of each scope', () => {
    expect(purgeCriteria(draft({ scope: 'thread', thread: 'a' }))).toEqual({ threads: ['a'] });
    expect(purgeCriteria(draft({ scope: 'idle', idle: 7 }))).toEqual({ idleDays: 7 });
    expect(purgeCriteria(draft({ scope: 'trash' }))).toEqual({ trash: true });
  });
});
