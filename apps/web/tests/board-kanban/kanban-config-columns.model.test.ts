/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { KanbanColumnConfig, KanbanConfig } from '../../src/data/types';
import { limitTone } from '../../src/screens/board/tabs/kanban/model';
import { HOUR, before, idsIn, itemRef, viewOf } from './kanban-docs';

const active: KanbanColumnConfig = { id: 'active', name: 'Active', accent: 'blue', match: { lanes: ['doing', 'review'] }, limit: { max: 1, mode: 'refuse' }, rules: ['stale'] };

const config: KanbanConfig = {
  columns: [
    { id: 'ready', name: 'Ready', accent: 'teal', match: { lanes: ['todo'], stages: ['ready'] } },
    { id: 'backlog', name: 'Backlog', match: { lanes: ['todo'] }, limit: { max: 2, mode: 'warn' } },
    active,
    { id: 'shipped', name: 'Shipped', match: { lanes: ['done', 'dropped'] } },
  ],
  rules: [{ id: 'stale', kind: 'dwell', params: { ms: HOUR } }],
};

const items = [
  itemRef('I-1', { status: 'todo', stage: 'ready' }),
  itemRef('I-2', { status: 'todo' }),
  itemRef('I-3', { status: 'doing' }),
  itemRef('I-4', { status: 'review' }),
  itemRef('I-5', { status: 'dropped' }),
];

describe('I-206 columns come from the board config', { tags: ['critical'] }, () => {
  it('puts an item in the first column whose lanes and stages take it', () => {
    const view = viewOf(items, { config, prefs: { showDone: true } });
    expect(view.columns.map((column) => column.id)).toEqual(['ready', 'backlog', 'active', 'shipped']);
    expect(idsIn(view, 'ready')).toEqual(['I-1']);
    expect(idsIn(view, 'backlog')).toEqual(['I-2']);
    expect(idsIn(view, 'active')).toEqual(['I-3', 'I-4']);
    expect(idsIn(view, 'shipped')).toEqual(['I-5']);
  });

  it('names the columns and passes the accent, dropping an unknown one', () => {
    const view = viewOf(items, { config: { columns: [{ id: 'a', name: 'Alpha', accent: 'mauve', match: { lanes: ['todo'] } }, active] } });
    expect(view.columns.map((column) => [column.title, column.accent])).toEqual([['Alpha', undefined], ['Active', 'blue']]);
  });

  it('hides a closed column until done items show', () => {
    expect(viewOf(items, { config }).columns.map((column) => column.id)).not.toContain('shipped');
  });

  it('leaves out an item no column takes', () => {
    const view = viewOf(items, { config: { columns: [active] } });
    expect(view.cards.length).toBe(4);
    expect(view.columns.flatMap((column) => column.cards).length).toBe(2);
  });

  it('keeps today columns with no config or an empty one', () => {
    const ids = (cfg?: KanbanConfig | null) => viewOf(items, { config: cfg }).columns.map((column) => column.id);
    expect(ids()).toEqual(['todo', 'doing', 'review']);
    expect(ids({ columns: [] })).toEqual(['todo', 'doing', 'review']);
  });

  it('maps a column to the first lane of its match for a drop', () => {
    const view = viewOf(items, { config });
    expect(view.targets.get('active')).toBe('doing');
    expect(view.targets.get('shipped')).toBe('done');
  });
});

describe('I-206 column limits', { tags: ['standard'] }, () => {
  it('carries the limits of the columns that have one', () => {
    const view = viewOf(items, { config });
    expect([...view.limits.keys()]).toEqual(['backlog', 'active']);
  });

  it('tones a limit by its mode only when the count is over', () => {
    expect(limitTone(2, { max: 2, mode: 'warn' })).toBe('info');
    expect(limitTone(3, { max: 2, mode: 'warn' })).toBe('warning');
    expect(limitTone(2, { max: 1, mode: 'refuse' })).toBe('danger');
    expect(limitTone(2, { max: 1, mode: 'alert' })).toBe('danger');
  });

  it('carries no limit when the cards are grouped by thread', () => {
    expect(viewOf(items, { config, prefs: { group: 'thread' } }).limits.size).toBe(0);
  });
});

describe('I-206 stale hours follow the config', { tags: ['standard'] }, () => {
  const old = [itemRef('I-1', { status: 'doing', updatedAt: before(2 * HOUR) })];

  it('uses the dwell rule of the column of the lane', () => {
    expect(viewOf(old, { config }).quiet).toBe(1);
  });

  it('keeps today hours with no config', () => {
    expect(viewOf(old).quiet).toBe(0);
  });
});
