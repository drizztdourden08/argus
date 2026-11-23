/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { epicColumns, epicFolded, epicKeyOf, epicKeys, epicStored, rulesText } from '../../src/screens/board/tabs/kanban/model';
import { itemRef, viewOf } from './kanban-docs';

const items = [
  itemRef('I-1', { type: 'epic', rank: 1 }),
  itemRef('I-2', { parent: 'I-1', rank: 2 }),
  itemRef('I-3', { rank: 3 }),
  itemRef('I-4', { parent: 'I-1', rank: 4 }),
  itemRef('I-5', { parent: 'I-1', rank: 5, status: 'doing' }),
];

const shownOf = () => {
  const view = viewOf(items, { prefs: { order: 'rank' } });
  return epicColumns(view.columns, view.parents);
};

describe('I-206 the epics view folds each column into one container per epic', { tags: ['critical'] }, () => {
  it('puts the children of an epic under one key per column, and the rest under No epic', () => {
    const todo = shownOf().find((column) => column.id === 'todo');
    expect(todo?.cards.map((card) => [card.id, card.group])).toEqual([
      ['I-2', 'epic:I-1:todo'],
      ['I-4', 'epic:I-1:todo'],
      ['I-3', 'epic:none:todo'],
    ]);
  });

  it('keys the same epic apart in another column', () => {
    const doing = shownOf().find((column) => column.id === 'doing');
    expect(doing?.cards.map((card) => card.group)).toEqual(['epic:I-1:doing']);
  });

  it('reads a key back into its epic and column', () => {
    expect(epicKeyOf('epic:I-1:todo')).toEqual({ epic: 'I-1', column: 'todo' });
    expect(epicKeyOf('epic:none:todo')).toEqual({ epic: null, column: 'todo' });
    expect(epicKeyOf('I-1')).toBeNull();
  });

  it('starts every container folded and records only the opened ones', () => {
    const keys = epicKeys(shownOf());
    expect(keys).toEqual(['epic:I-1:todo', 'epic:none:todo', 'epic:I-1:doing']);
    expect(epicFolded(['I-9'], keys)).toEqual(['I-9', ...keys]);
    expect(epicFolded(['epic:I-1:todo'], keys)).toEqual(['epic:none:todo', 'epic:I-1:doing']);
    expect(epicStored(['I-9', 'epic:I-1:todo'], keys, ['epic:none:todo', 'epic:I-1:doing'])).toEqual(['I-9', 'epic:I-1:todo']);
  });
});

describe('I-206 the rules count reads in the singular', { tags: ['critical'] }, () => {
  it('says 1 rule, 0 rules and 2 rules', () => {
    expect([rulesText(0), rulesText(1), rulesText(2)]).toEqual(['0 rules', '1 rule', '2 rules']);
  });
});
