/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { boardEditOp } from '../../src/shell/records/pane/board/model/board-edit-op';
import { depsOfBoard } from '../../src/shell/records/pane/board/model/deps-of-board';
import { failingTests } from '../../src/shell/records/pane/board/model/failing-tests';
import { openItems } from '../../src/shell/records/pane/board/model/open-items';
import { record } from '../overview-boards/helpers';

const item = (id: string, status: string, extra: object = {}) => ({ id, title: id, status, thread: 't', ...extra });

const laneIds = (group: { lane: string; items: { id: string }[] }) => [group.lane, group.items.map((entry) => entry.id)];

describe('I-112 P5 the board pane model', { tags: ['critical'] }, () => {
  it('builds an edit of the goal with the previous text and the base revision', () => {
    const board = { ...record('argus', {}, { goal: 'Ship it', rev: 4 }), id: 'BD-1' };
    expect(boardEditOp(board, 'goal', 'Ship it now')).toEqual({ kind: 'edit', id: 'BD-1', field: 'goal', value: 'Ship it now', prev: 'Ship it', baseRev: 4 });
  });

  it('sends a null previous text when the note was empty', () => {
    const board = record('argus');
    expect(boardEditOp(board, 'note', 'First')).toMatchObject({ prev: null });
  });

  it('groups open items by lane and leaves out done, archived and deleted ones', () => {
    const groups = openItems([item('I-1', 'doing'), item('I-2', 'todo'), item('I-3', 'done'), item('I-4', 'review', { archivedAt: 'x' }), item('I-5', 'doing', { deletedAt: 'x' })]);
    expect(groups.map(laneIds)).toEqual([
      ['doing', ['I-1']],
      ['todo', ['I-2']],
    ]);
  });

  it('keeps only failing tests that are still live', () => {
    const base = { item: null, thread: null, covers: null, mode: 'auto', run: null, context: null, ac: [], createdAt: '', updatedAt: '', result: null };
    const tests = [
      { id: 'T-1', title: 'a', status: 'fail', ...base },
      { id: 'T-2', title: 'b', status: 'pass', ...base },
      { id: 'T-3', title: 'c', status: 'fail', deletedAt: 'x', ...base },
    ] as never;
    expect(failingTests(tests).map((test) => test.id)).toEqual(['T-1']);
  });

  it('splits dependencies into those asked of the board and those it waits on', () => {
    const dep = (id: string, from: string, to: string) => ({ id, from: { project: from, item: null }, to: { project: to, item: null } });
    const sides = depsOfBoard([dep('D-1', 'argus', 'tessera'), dep('D-2', 'brock', 'argus'), dep('D-3', 'brock', 'tessera')] as never, 'argus');
    expect(sides.outgoing.map((entry) => entry.id)).toEqual(['D-1']);
    expect(sides.incoming.map((entry) => entry.id)).toEqual(['D-2']);
  });
});
