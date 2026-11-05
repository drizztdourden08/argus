/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { boardEntry } from '../../src/data/boards';
import { parseRecord } from '../../src/shell/records/behavior/parse-record';

describe('P3 the board data layer', { tags: ['critical'] }, () => {
  it('builds an edit of goal or note against the boards collection', () => {
    expect(boardEntry({ kind: 'edit', id: 'BD-3', field: 'goal', value: 'new', prev: 'old', baseRev: 2 })).toEqual({
      kind: 'edit',
      target: { c: 'boards', id: 'BD-3' },
      set: { goal: 'new' },
      prev: { goal: 'old' },
      baseRev: 2,
    });
  });

  it('builds archive and unarchive with a target only', () => {
    expect(boardEntry({ kind: 'archive', id: 'BD-3' })).toEqual({ kind: 'archive', target: { c: 'boards', id: 'BD-3' } });
    expect(boardEntry({ kind: 'unarchive', id: 'BD-3' })).toEqual({ kind: 'unarchive', target: { c: 'boards', id: 'BD-3' } });
  });

  it('pins BD-3 to the portfolio scope, even when written on another board', () => {
    expect(parseRecord('BD-3')).toMatchObject({ kind: 'board', collection: 'boards', code: 'BD-3', id: 'BD-3', board: '_portfolio' });
    expect(parseRecord('argus/BD-3')?.board).toBe('_portfolio');
  });

  it('leaves other kinds on their own board', () => {
    expect(parseRecord('argus/I-3')?.board).toBe('argus');
  });
});
