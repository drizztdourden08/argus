/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { boardProgress } from '../../src/screens/overview/boards/model/board-progress';
import { COUNTS, record } from './helpers';

describe('P3 the progress of a board, done over total', { tags: ['critical'] }, () => {
  it('splits the open items into review, doing and todo', () => {
    const found = boardProgress(record('a', { counts: { ...COUNTS, done: 6, itemsOpen: 4, doing: 1, review: 1 } }));
    expect(found).toEqual({ done: 6, review: 1, doing: 1, todo: 2, total: 10, ratio: 0.6 });
  });

  it('is zero with no items and never divides by zero', () => {
    expect(boardProgress(record('a', null))).toMatchObject({ total: 0, ratio: 0, todo: 0 });
  });

  it('never goes below zero when doing and review exceed the open items', () => {
    expect(boardProgress(record('a', { counts: { ...COUNTS, itemsOpen: 1, doing: 2, review: 1 } })).todo).toBe(0);
  });

  it('is complete when nothing is open', () => {
    expect(boardProgress(record('a', { counts: { ...COUNTS, done: 3 } })).ratio).toBe(1);
  });
});
