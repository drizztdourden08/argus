/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import {
  bulkMoves, bulkSofts, formatFilters, noticeRows, opNotices, parseStored, planDrop, textProblem,
} from '../../src/screens/board/tabs/kanban/model';
import type { DropInput, ItemGroup, ItemOrder } from '../../src/screens/board/tabs/kanban/model';
import { entryRef, filters, itemRef, prefs, threadRef, viewOf } from './kanban-docs';

const todo = [itemRef('I-1', { rank: 1000 }), itemRef('I-2', { rank: 2000 })];
const doing = [itemRef('I-3', { status: 'doing' })];

const planOf = (move: DropInput['move'], group: ItemGroup = 'status', order: ItemOrder = 'rank', items = [...todo, ...doing]) => {
  const view = viewOf(items, { prefs: { group, order } });
  return planDrop({ columns: view.columns, move, group, order });
};

const across = (from: string, to: string, index: number) => ({ cardId: 'I-1', from: { columnId: from, index: 0 }, to: { columnId: to, index } });

describe('where a drop puts a card', () => {
  it('reads the neighbour in the new column: before the card it lands above', () => {
    expect(planOf(across('todo', 'doing', 0))).toEqual({ kind: 'status', to: 'doing', place: { before: 'I-3' } });
  });

  it('reads the neighbour in the new column: after the card it lands below', () => {
    expect(planOf(across('todo', 'doing', 1))).toEqual({ kind: 'status', to: 'doing', place: { after: 'I-3' } });
  });

  it('gives no place in an empty column', () => {
    expect(planOf(across('todo', 'review', 0))).toEqual({ kind: 'status', to: 'review', place: {} });
  });

  it('gives no place when the order is by update, only the new column', () => {
    expect(planOf(across('todo', 'doing', 0), 'status', 'updated')).toEqual({ kind: 'status', to: 'doing', place: {} });
  });

  it('reorders inside a column from the neighbour it lands next to', () => {
    const move = { cardId: 'I-1', from: { columnId: 'todo', index: 0 }, to: { columnId: 'todo', index: 1 } };
    expect(planOf(move)).toEqual({ kind: 'reorder', place: { after: 'I-2' } });
    const back = { cardId: 'I-2', from: { columnId: 'todo', index: 1 }, to: { columnId: 'todo', index: 0 } };
    expect(planOf(back)).toEqual({ kind: 'reorder', place: { before: 'I-1' } });
  });

  it('says to switch to Manual for a reorder when the order is by update', () => {
    const move = { cardId: 'I-1', from: { columnId: 'todo', index: 0 }, to: { columnId: 'todo', index: 1 } };
    expect(planOf(move, 'status', 'updated')).toEqual({ kind: 'order-off' });
  });

  it('does nothing for a drop where it was', () => {
    const move = { cardId: 'I-1', from: { columnId: 'todo', index: 0 }, to: { columnId: 'todo', index: 0 } };
    expect(planOf(move)).toEqual({ kind: 'none' });
  });

  it('reassigns for a thread column and sets the subject for a subject column', () => {
    const items = [itemRef('I-1'), itemRef('I-2', { thread: 'other', subject: 'ui' })];
    expect(planOf(across('worker', 'other', 0), 'thread', 'rank', items)).toEqual({ kind: 'thread', to: 'other', place: { before: 'I-2' } });
    expect(planOf(across('general', 'ui', 0), 'subject', 'rank', items)).toEqual({ kind: 'subject', to: 'ui', place: { before: 'I-2' } });
  });
});

describe('what the board refused', () => {
  const move = { kind: 'move', target: { c: 'items', id: 'I-1' }, to: { status: 'doing' }, prev: { thread: 'worker', status: 'todo' }, baseRev: 3 };

  it('lists a conflicted op with the thread that changed it', () => {
    const entries = [entryRef('O-1', { author: 'human', ...move, status: 'conflict', by: 'worker', current: { status: 'review' }, item: null })];
    const [notice] = opNotices(entries, new Set());
    expect(notice).toMatchObject({ id: 'O-1', entryId: 'O-1', bulk: false, kind: 'conflict', result: { by: 'worker', current: { status: 'review' } } });
  });

  it('lists a refused op with its reason', () => {
    const entries = [entryRef('O-1', { kind: 'edit', author: 'human', target: { c: 'items', id: 'I-1' }, set: { title: 'x' }, status: 'refused', outcome: 'not yours', item: null })];
    expect(opNotices(entries, new Set())).toMatchObject([{ kind: 'refused', result: { outcome: 'not yours' } }]);
  });

  it('skips what the board applied, what is still on its way, what the owner dismissed and ops on other collections', () => {
    const entries = [
      entryRef('O-1', { author: 'human', ...move, status: 'applied', item: null }),
      entryRef('O-2', { author: 'human', ...move, status: 'new', item: null }),
      entryRef('O-3', { author: 'human', ...move, status: 'conflict', item: null }),
      entryRef('O-4', { kind: 'ack', author: 'human', replyTo: 'O-3', item: null }),
      entryRef('O-5', { kind: 'edit', author: 'human', target: { c: 'agents', id: 'A-1' }, status: 'refused', item: null }),
      entryRef('O-6', { ...move, author: 'worker', status: 'conflict', item: null }),
    ];
    expect(opNotices(entries, new Set())).toEqual([]);
  });

  it('fans a bulk entry out, one notice for each op the board did not take', () => {
    const entries = [
      entryRef('O-1', {
        kind: 'bulk',
        author: 'human',
        item: null,
        ops: [move, { ...move, target: { c: 'items', id: 'I-2' } }, { ...move, target: { c: 'items', id: 'I-3' } }],
        results: [{ status: 'applied' }, { status: 'conflict', current: { status: 'done' }, by: 'lead' }, { status: 'refused', outcome: 'locked' }],
        status: 'applied',
      }),
    ];
    const notices = opNotices(entries, new Set());
    expect(notices.map((notice) => [notice.id, notice.kind, notice.bulk])).toEqual([['O-1#1', 'conflict', true], ['O-1#2', 'refused', true]]);
    expect(opNotices(entries, new Set(['O-1#1'])).map((notice) => notice.id)).toEqual(['O-1#2']);
  });

  it('shows what the owner meant next to what is there now', () => {
    const [notice] = opNotices([entryRef('O-1', { author: 'human', ...move, to: { status: 'doing', before: 'I-3' }, status: 'conflict', current: { status: 'review' }, item: null })], new Set());
    expect(notice && noticeRows(notice)).toEqual([{ field: 'status', mine: 'doing', theirs: 'review' }]);
  });

  it('shows the text of a criterion the owner meant', () => {
    const [notice] = opNotices([entryRef('O-1', { kind: 'ac', author: 'human', target: { c: 'items', id: 'I-1' }, do: 'set', n: 1, text: 'mine', status: 'conflict', current: { text: 'theirs' }, item: null })], new Set());
    expect(notice && noticeRows(notice)).toEqual([{ field: 'text', mine: 'mine', theirs: 'theirs' }]);
  });
});

describe('what the page remembers', () => {
  it('reads the choices of the Node page: filters with the order, the grouping and the done toggle', () => {
    const stored = parseStored({
      filters: JSON.stringify({ q: 'log', threads: ['worker'], status: 'doing', subject: 'ui', needs: true, conf: true, archived: true, sort: 'updated', builds: ['b-1'] }),
      group: 'thread',
      showDone: '1',
    });
    expect(stored.filters).toEqual({ query: 'log', threads: ['worker'], status: 'doing', subject: 'ui', needs: true, conflict: true, archived: true });
    expect(stored.prefs).toEqual({ group: 'thread', order: 'updated', showDone: true });
  });

  it('falls back to the defaults for nothing, for junk and for a grouping it does not know', () => {
    expect(parseStored({ filters: null, group: null, showDone: null })).toEqual({ filters: filters(), prefs: prefs() });
    expect(parseStored({ filters: '{not json', group: 'build', showDone: '0' })).toEqual({ filters: filters(), prefs: prefs() });
  });

  it('writes the filters back without losing the fields of the Tests tab', () => {
    const written = formatFilters(JSON.stringify({ builds: ['b-1'], nobuild: true }), filters({ query: 'x', conflict: true }), 'updated');
    expect(JSON.parse(written)).toMatchObject({ builds: ['b-1'], nobuild: true, q: 'x', conf: true, sort: 'updated', status: '' });
  });
});

describe('the limits of an edit', () => {
  it('needs a title and a criterion, and allows the rest to be empty', () => {
    expect(textProblem('title', '  ')).toBe('That cannot be empty.');
    expect(textProblem('criterion', '')).toBe('A criterion cannot be empty.');
    expect(textProblem('subject', '')).toBeNull();
    expect(textProblem('description', '')).toBeNull();
  });

  it('stops a text over its limit: 80 for a title, 40 for a subject, 400 for a description, 120 for a criterion', () => {
    expect(textProblem('title', 'x'.repeat(80))).toBeNull();
    expect(textProblem('title', 'x'.repeat(81))).toBe('At most 80 characters.');
    expect(textProblem('subject', 'x'.repeat(41))).toBe('At most 40 characters.');
    expect(textProblem('description', 'x'.repeat(401))).toBe('At most 400 characters.');
    expect(textProblem('criterion', 'x'.repeat(121))).toBe('At most 120 characters.');
  });
});

describe('the bulk ops', () => {
  const picked = [itemRef('I-1', { rev: 4 }), itemRef('I-2', { thread: 'other', status: 'doing', rev: 7 })];

  it('moves each with what it was and its revision', () => {
    expect(bulkMoves(picked, { status: 'review' })).toEqual([
      { kind: 'move', target: { c: 'items', id: 'I-1' }, to: { status: 'review' }, prev: { thread: 'worker', status: 'todo' }, baseRev: 4 },
      { kind: 'move', target: { c: 'items', id: 'I-2' }, to: { status: 'review' }, prev: { thread: 'other', status: 'doing' }, baseRev: 7 },
    ]);
  });

  it('pairs archive with unarchive and delete with restore for the Undo', () => {
    expect(bulkSofts('archive', picked)).toEqual({
      ops: [{ kind: 'archive', target: { c: 'items', id: 'I-1' } }, { kind: 'archive', target: { c: 'items', id: 'I-2' } }],
      inverse: [{ kind: 'unarchive', target: { c: 'items', id: 'I-1' } }, { kind: 'unarchive', target: { c: 'items', id: 'I-2' } }],
    });
    expect(bulkSofts('delete', picked).inverse.map((op) => op.kind)).toEqual(['restore', 'restore']);
  });
});

describe('the thread seeds', () => {
  it('sees a thread archived by its date as hidden', () => {
    const view = viewOf([], { prefs: { group: 'thread' }, threads: [threadRef('a'), threadRef('b', { archivedAt: '2026-10-01T00:00:00.000Z' })] });
    expect(view.columns.map((column) => column.id)).toEqual(['a']);
  });
});
