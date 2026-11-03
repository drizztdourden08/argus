/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { ItemRef } from '../../src/data/kanban';
import { countsText, filtersActive, itemTests, needsMap } from '../../src/screens/board/tabs/kanban/model';
import type { OpNotice } from '../../src/screens/board/shared';
import type { Needs } from '../../src/screens/board/tabs/kanban/model';
import { DAY, HOUR, before, entryRef, filters, idsIn, itemRef, noNeeds, threadRef, viewOf } from './kanban-docs';

const quietOf = (item: ItemRef, needs: ReadonlyMap<string, Needs> = noNeeds): number | null =>
  viewOf([item], { prefs: { showDone: true }, needs }).cards[0]?.quietMs ?? null;

describe('K1 the quiet limits', { tags: ['critical'] }, () => {
  it('marks a doing item after 4 hours, a review item after 6 and a to-do after 24', () => {
    expect(quietOf(itemRef('I-1', { status: 'doing', updatedAt: before(5 * HOUR) }))).toBe(5 * HOUR);
    expect(quietOf(itemRef('I-2', { status: 'doing', updatedAt: before(3 * HOUR) }))).toBeNull();
    expect(quietOf(itemRef('I-3', { status: 'review', updatedAt: before(7 * HOUR) }))).toBe(7 * HOUR);
    expect(quietOf(itemRef('I-4', { status: 'review', updatedAt: before(5 * HOUR) }))).toBeNull();
    expect(quietOf(itemRef('I-5', { status: 'todo', updatedAt: before(2 * DAY) }))).toBe(2 * DAY);
    expect(quietOf(itemRef('I-6', { status: 'todo', updatedAt: before(23 * HOUR) }))).toBeNull();
  });

  it('counts the limit itself as quiet', () => {
    expect(quietOf(itemRef('I-1', { status: 'doing', updatedAt: before(4 * HOUR) }))).toBe(4 * HOUR);
  });

  it('never marks a done, dropped or archived item', () => {
    expect(quietOf(itemRef('I-1', { status: 'done', updatedAt: before(9 * DAY) }))).toBeNull();
    expect(quietOf(itemRef('I-2', { status: 'dropped', updatedAt: before(9 * DAY) }))).toBeNull();
    expect(quietOf(itemRef('I-3', { status: 'doing', updatedAt: before(9 * DAY), archivedAt: before(DAY) }))).toBeNull();
  });

  it('does not mark an item that waits on the owner', () => {
    const needs = new Map([['I-1', { questions: 1, decisions: 0, todos: 0, tests: 0 }]]);
    expect(quietOf(itemRef('I-1', { status: 'doing', updatedAt: before(2 * DAY) }), needs)).toBeNull();
  });

  it('counts the quiet cards that show', () => {
    const view = viewOf([
      itemRef('I-1', { status: 'doing', updatedAt: before(5 * HOUR) }),
      itemRef('I-2', { status: 'todo', updatedAt: before(2 * DAY) }),
      itemRef('I-3', { status: 'doing', updatedAt: before(HOUR) }),
    ]);
    expect(view.quiet).toBe(2);
    expect(countsText(view)).toBe('3 open / 3');
  });
});

describe('K1 what waits on the owner', { tags: ['critical'] }, () => {
  it('counts an open question, a pending decision, a pending to-do and a manual test to run', () => {
    const map = needsMap({
      entries: [entryRef('Q-1'), entryRef('D-1', { kind: 'decision', status: 'pending' }), entryRef('Q-2', { status: 'done' })],
      actions: [{ id: 'A-1', item: 'I-1', status: 'pending' }, { id: 'A-2', item: 'I-1', status: 'done' }],
      tests: [{ id: 'T-1', item: 'I-1', mode: 'manual', status: 'pending' }, { id: 'T-2', item: 'I-1', mode: 'auto', status: 'pending' }],
    });
    expect(map.get('I-1')).toEqual({ questions: 1, decisions: 1, todos: 1, tests: 1 });
  });

  it('does not count a question the owner answered or dismissed', () => {
    const map = needsMap({
      entries: [
        entryRef('Q-1'),
        entryRef('Q-2'),
        entryRef('H-1', { kind: 'followup', author: 'human', replyTo: 'Q-1', item: null }),
        entryRef('H-2', { kind: 'ack', author: 'human', replyTo: 'Q-2', item: null }),
      ],
      actions: [],
      tests: [],
    });
    expect(map.get('I-1')).toBeUndefined();
  });

  it('does not count a to-do the owner has just marked done', () => {
    const map = needsMap({
      entries: [entryRef('H-1', { kind: 'done', author: 'human', replyTo: 'A-1', item: null, status: 'new' })],
      actions: [{ id: 'A-1', item: 'I-1', status: 'pending' }],
      tests: [],
    });
    expect(map.get('I-1')).toBeUndefined();
  });
});

describe('K1 the board columns', { tags: ['critical'] }, () => {
  const finished = [itemRef('I-8', { status: 'done' }), itemRef('I-9', { status: 'dropped' })];
  const all = [itemRef('I-1', { status: 'doing' }), ...finished];

  it('hides done and dropped items by default and keeps counting the outstanding ones', () => {
    const view = viewOf(all);
    expect(view.columns.map((column) => column.id)).toEqual(['todo', 'doing', 'review']);
    expect(view.cards.map((card) => card.id)).toEqual(['I-1']);
    expect(view.closed).toBe(2);
    expect(countsText(view)).toBe('1 open / 3');
  });

  it('shows them, dropped ones in the Done column, once the choice is on', () => {
    const view = viewOf(all, { prefs: { showDone: true } });
    expect(view.columns.map((column) => column.id)).toEqual(['todo', 'doing', 'review', 'done']);
    expect(idsIn(view, 'done')).toEqual(['I-8', 'I-9']);
  });

  it('shows done and dropped items for a status filter on them without the choice', () => {
    const view = viewOf(all, { filters: { status: 'dropped' } });
    expect(view.cards.map((card) => card.id)).toEqual(['I-9']);
    expect(view.columns.map((column) => column.id)).toContain('done');
  });

  it('filters by id or title, ignoring case', () => {
    const items = [itemRef('I-1', { title: 'Login page' }), itemRef('I-2', { title: 'Logout' }), itemRef('I-3', { title: 'Other' })];
    expect(viewOf(items, { filters: { query: 'LOG' } }).cards.map((card) => card.id)).toEqual(['I-1', 'I-2']);
    expect(viewOf(items, { filters: { query: 'i-3' } }).cards.map((card) => card.id)).toEqual(['I-3']);
  });

  it('filters by thread, needs you and conflict', () => {
    const items = [itemRef('I-1'), itemRef('I-2', { thread: 'other' })];
    expect(viewOf(items, { filters: { threads: ['other'] } }).cards.map((card) => card.id)).toEqual(['I-2']);
    const needs = new Map([['I-1', { questions: 1, decisions: 0, todos: 0, tests: 0 }]]);
    expect(viewOf(items, { filters: { needs: true }, needs }).cards.map((card) => card.id)).toEqual(['I-1']);
    const notices: OpNotice[] = [{ id: 'O-1', entryId: 'O-1', bulk: false, kind: 'conflict', op: { kind: 'move', target: { c: 'items', id: 'I-2' } }, result: {} }];
    expect(viewOf(items, { filters: { conflict: true }, notices }).cards.map((card) => card.id)).toEqual(['I-2']);
  });

  it('filters by type, an item without a type being a task', () => {
    const items = [itemRef('I-1'), itemRef('I-2', { type: 'bug' }), itemRef('I-3', { type: 'epic' })];
    expect(viewOf(items, { filters: { types: ['bug'] } }).cards.map((card) => card.id)).toEqual(['I-2']);
    expect(viewOf(items, { filters: { types: ['task', 'epic'] } }).cards.map((card) => card.id)).toEqual(['I-1', 'I-3']);
  });

  it('filters by assignee, an item without one being held by its thread', () => {
    const items = [itemRef('I-1'), itemRef('I-2', { assignee: 'owner' }), itemRef('I-3', { assignee: 'lead' })];
    expect(viewOf(items, { filters: { assignees: ['owner'] } }).cards.map((card) => card.id)).toEqual(['I-2']);
    expect(viewOf(items, { filters: { assignees: ['worker', 'lead'] } }).cards.map((card) => card.id)).toEqual(['I-1', 'I-3']);
    expect(filtersActive(filters({ types: ['bug'] }))).toBe(true);
    expect(filtersActive(filters({ assignees: ['owner'] }))).toBe(true);
  });

  it('marks the cards that have a conflict', () => {
    const notices: OpNotice[] = [{ id: 'O-1', entryId: 'O-1', bulk: false, kind: 'conflict', op: { kind: 'edit', target: { c: 'items', id: 'I-1' } }, result: {} }];
    expect(viewOf([itemRef('I-1')], { notices }).cards[0]?.conflicted).toBe(true);
  });

  it('keeps archived items out unless asked, and deleted ones always', () => {
    const items = [itemRef('I-1'), itemRef('I-2', { archivedAt: before(HOUR) }), itemRef('I-3', { deletedAt: before(HOUR) })];
    expect(viewOf(items).cards.map((card) => card.id)).toEqual(['I-1']);
    expect(viewOf(items, { filters: { archived: true } }).cards.map((card) => card.id)).toEqual(['I-1', 'I-2']);
  });

  it('orders a column by rank, or by the newest update', () => {
    const items = [
      itemRef('I-1', { rank: 2000, updatedAt: before(HOUR) }),
      itemRef('I-2', { rank: 1000, updatedAt: before(3 * HOUR) }),
      itemRef('I-3', { rank: 3000, updatedAt: before(2 * HOUR) }),
    ];
    expect(idsIn(viewOf(items), 'todo')).toEqual(['I-2', 'I-1', 'I-3']);
    expect(idsIn(viewOf(items, { prefs: { order: 'updated' } }), 'todo')).toEqual(['I-1', 'I-3', 'I-2']);
  });

  it('knows when a filter is on', () => {
    expect(filtersActive(filters())).toBe(false);
    expect(filtersActive(filters({ archived: true }))).toBe(false);
    expect(filtersActive(filters({ needs: true }))).toBe(true);
    expect(countsText(viewOf([itemRef('I-1')], { filters: { query: 'x' } }))).toBe('1 open / 1 · filtered');
  });
});

describe('K1 the other groupings', { tags: ['critical'] }, () => {
  it('makes a column per thread, an empty one for a listed thread, none for the coordinator or a placeholder', () => {
    const threads = [
      threadRef('worker'),
      threadRef('idle'),
      threadRef('lead', { role: 'coordinator' }),
      threadRef('ghost', { placeholder: true }),
      threadRef('old', { status: 'archived' }),
    ];
    const view = viewOf([itemRef('I-1', { thread: 'zed' })], { prefs: { group: 'thread' }, threads });
    expect(view.columns.map((column) => column.id)).toEqual(['idle', 'worker', 'zed']);
    expect(idsIn(view, 'zed')).toEqual(['I-1']);
  });

  it('leaves the empty thread columns out while a filter is on', () => {
    const threads = [threadRef('worker'), threadRef('idle')];
    const view = viewOf([itemRef('I-1')], { prefs: { group: 'thread' }, threads, filters: { query: 'I-1' } });
    expect(view.columns.map((column) => column.id)).toEqual(['worker']);
  });

  it('makes a column per assignee, the owner first, an item without one under its thread', () => {
    const items = [itemRef('I-1', { assignee: 'owner' }), itemRef('I-2', { assignee: 'zed' }), itemRef('I-3'), itemRef('I-4', { assignee: 'worker' })];
    const view = viewOf(items, { prefs: { group: 'assignee' } });
    expect(view.columns.map((column) => column.id)).toEqual(['owner', 'worker', 'zed']);
    expect(view.columns.map((column) => column.title)).toEqual(['You', 'worker', 'zed']);
    expect(idsIn(view, 'worker')).toEqual(['I-3', 'I-4']);
  });

  it('makes a column per parent, titled with the parent, and one for the items without a parent last', () => {
    const items = [
      itemRef('I-1', { type: 'epic', title: 'Rework' }),
      itemRef('I-2', { parent: 'I-1', type: 'story' }),
      itemRef('I-3', { parent: 'I-1' }),
      itemRef('I-4'),
    ];
    const view = viewOf(items, { prefs: { group: 'parent' } });
    expect(view.columns.map((column) => column.id)).toEqual(['I-1', 'no-parent']);
    expect(view.columns.map((column) => column.title)).toEqual(['I-1 Rework', 'No parent']);
    expect(idsIn(view, 'I-1')).toEqual(['I-2', 'I-3']);
    expect(idsIn(view, 'no-parent')).toEqual(['I-1', 'I-4']);
  });
});

describe('K1 the tests a drawer counts', { tags: ['critical'] }, () => {
  const docs = [
    { id: 'T-1' },
    { id: 'T-2', archivedAt: '2026-10-25T21:37:00.080Z' },
    { id: 'T-3', deletedAt: '2026-10-25T21:37:00.080Z' },
    { id: 'T-4', build: 'old-1' },
    { id: 'T-5', build: 'open-2' },
  ];
  const closed = new Set(['old-1']);

  it('leaves out deleted and archived tests and those of a closed bundle', () => {
    expect(itemTests(docs, closed, false).map((test) => test.id)).toEqual(['T-1', 'T-5']);
  });

  it('keeps the archived ones while the archived filter is on, never the deleted or the closed', () => {
    expect(itemTests(docs, closed, true).map((test) => test.id)).toEqual(['T-1', 'T-2', 'T-5']);
  });
});
