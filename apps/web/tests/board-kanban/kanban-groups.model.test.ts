/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { familyItems, idsIn, itemRef, viewOf } from './kanban-docs';

const family = familyItems();

describe('I-94 children are framed under their parent', { tags: ['standard'] }, () => {
  it('keeps the children of one parent together where the first child sits', () => {
    const view = viewOf(family, { prefs: { order: 'rank' } });
    expect(idsIn(view, 'todo')).toEqual(['I-2', 'I-4', 'I-3']);
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

describe('I-209 an epic is shown once, as the container of its children', { tags: ['standard'] }, () => {
  it('never lists an id both as a card and as a group key in a column', () => {
    const view = viewOf(family, { prefs: { order: 'rank' } });
    for (const column of view.columns) {
      const keys = new Set(column.cards.flatMap((card) => (card.group ? [card.group] : [])));
      expect(column.cards.filter((card) => keys.has(card.id))).toEqual([]);
    }
  });

  it('shows an epic with no shown children as one ordinary card', () => {
    const view = viewOf([itemRef('I-1', { type: 'epic', rank: 1 }), itemRef('I-3', { rank: 3 })], { prefs: { order: 'rank' } });
    expect(idsIn(view, 'todo')).toEqual(['I-1', 'I-3']);
  });

  it('uses the root epic as the one container and puts each story before its tasks', () => {
    const tree = [
      itemRef('I-1', { type: 'epic', rank: 1 }),
      itemRef('I-2', { parent: 'I-1', rank: 2 }),
      itemRef('I-3', { parent: 'I-1', rank: 3 }),
      itemRef('I-4', { parent: 'I-3', rank: 4 }),
      itemRef('I-5', { parent: 'I-2', rank: 5 }),
    ];
    const view = viewOf(tree, { prefs: { order: 'rank' } });
    expect(idsIn(view, 'todo')).toEqual(['I-2', 'I-5', 'I-3', 'I-4']);
    expect(view.cards.filter((card) => card.id !== 'I-1').every((card) => card.group === 'I-1')).toBe(true);
  });
});
