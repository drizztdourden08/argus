/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { idsIn, itemRef, viewOf } from './kanban-docs';

const family = [
  itemRef('I-1', { type: 'epic', rank: 1 }),
  itemRef('I-2', { parent: 'I-1', rank: 2 }),
  itemRef('I-3', { rank: 3 }),
  itemRef('I-4', { parent: 'I-1', rank: 4 }),
];

describe('I-94 children are framed under their parent', { tags: ['standard'] }, () => {
  it('keeps the children of one parent together where the first child sits', () => {
    const view = viewOf(family, { prefs: { order: 'rank' } });
    expect(idsIn(view, 'todo')).toEqual(['I-1', 'I-2', 'I-4', 'I-3']);
  });

  it('gives each child the id of its parent as group, and leaves the others without', () => {
    const view = viewOf(family, { prefs: { order: 'rank' } });
    expect(view.cards.map((card) => card.group)).toEqual([undefined, 'I-1', undefined, 'I-1']);
  });

  it('sets no group when the columns are the parents', () => {
    const view = viewOf(family, { prefs: { order: 'rank', group: 'parent' } });
    expect(view.cards.every((card) => card.group === undefined)).toBe(true);
  });
});
