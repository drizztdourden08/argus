/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { attentionGroup } from '../../src/screens/overview/boards/model/attention-group';
import { groupBoards } from '../../src/screens/overview/boards/model/group-boards';
import { COUNTS, record } from './helpers';

const ARCHIVED = '2026-11-01T00:00:00.000Z';

const keysOf = (group: { boards: readonly { key: string }[] }): string[] => group.boards.map((entry) => entry.key);
const shapeOf = (group: { id: string; label: string; boards: readonly { key: string }[] }) => [group.id, group.label, keysOf(group)];
const idKeysOf = (group: { id: string; boards: readonly { key: string }[] }) => [group.id, keysOf(group)];

describe('P3 the attention group of a board', { tags: ['critical'] }, () => {
  it('puts a board that needs you, fails, is blocked, stalls or faults in Needs you', () => {
    expect(attentionGroup(record('a', { counts: { ...COUNTS, needsYou: 1 } }))).toBe('needs-you');
    expect(attentionGroup(record('a', { counts: { ...COUNTS, testsFailing: 1 } }))).toBe('needs-you');
    expect(attentionGroup(record('a', { counts: { ...COUNTS, blocked: 1 } }))).toBe('needs-you');
    expect(attentionGroup(record('a'), { fault: null, stalls: 1, syncDue: false })).toBe('needs-you');
    expect(attentionGroup(record('a'), { fault: { since: 's', reason: 'x' }, stalls: 0, syncDue: false })).toBe('needs-you');
  });

  it('puts a working board in Running and an idle one in Quiet', () => {
    expect(attentionGroup(record('a', { counts: { ...COUNTS, agents: 1 } }))).toBe('running');
    expect(attentionGroup(record('a'))).toBe('quiet');
  });

  it('puts a board with no card or board in Not started, and an archived one in Archived above all', () => {
    expect(attentionGroup(record('a', null))).toBe('not-started');
    expect(attentionGroup(record('a', { state: 'not started', lead: undefined }))).toBe('not-started');
    expect(attentionGroup(record('a', { counts: { ...COUNTS, needsYou: 2 } }, { archivedAt: ARCHIVED }))).toBe('archived');
  });
});

describe('P3 grouping boards by attention', { tags: ['critical'] }, () => {
  const boards = [
    record('quiet'),
    record('gone', null, { archivedAt: ARCHIVED }),
    record('need', { counts: { ...COUNTS, needsYou: 1 } }),
    record('new', null),
    record('busy', { counts: { ...COUNTS, agents: 2 } }),
  ];

  it('orders the groups Needs you, Running, Quiet, Not started, Archived', () => {
    const groups = groupBoards(boards);
    expect(groups.map(shapeOf)).toEqual([
      ['needs-you', 'Needs you', ['need']],
      ['running', 'Running', ['busy']],
      ['quiet', 'Quiet', ['quiet']],
      ['not-started', 'Not started', ['new']],
      ['archived', 'Archived', ['gone']],
    ]);
  });

  it('folds the last two groups and leaves the others open', () => {
    expect(groupBoards(boards).map((group) => group.folded)).toEqual([false, false, false, true, true]);
  });

  it('leaves out empty groups and keeps the input order inside a group', () => {
    const groups = groupBoards([record('b'), record('a')]);
    expect(groups.map((group) => group.id)).toEqual(['quiet']);
    expect(groups[0] && keysOf(groups[0])).toEqual(['b', 'a']);
  });

  it('reads the live facts of each board through the lookup', () => {
    const groups = groupBoards([record('a'), record('b')], (entry) => ({ fault: null, stalls: entry.key === 'b' ? 1 : 0, syncDue: false }));
    expect(groups.map(idKeysOf)).toEqual([['needs-you', ['b']], ['quiet', ['a']]]);
  });
});
