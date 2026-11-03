/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { REGISTRY_KINDS, kindOf } from '../../src/data/registry';
import { assigneeOfRecord } from '../../src/shell/records/behavior/assignee-of-record';
import { groupKeysOf } from '../../src/shell/records/behavior/group-keys-of';
import { groupRecords } from '../../src/shell/records/behavior/group-records';
import { legacyOpen } from '../../src/shell/records/behavior/legacy-open';
import { openOfHref } from '../../src/shell/records/behavior/open-of-href';
import { parseRecord } from '../../src/shell/records/behavior/parse-record';
import { recordHref } from '../../src/shell/records/behavior/record-href';
import { recordMenuGroups } from '../../src/shell/records/behavior/record-menu-groups';
import { recordSearch } from '../../src/shell/records/behavior/record-search';
import { shownCode } from '../../src/shell/records/behavior/shown-code';
import { RECORD_ACTIONS } from '../../src/shell/records/record-actions.constants';

const read = (token: string) => {
  const ref = parseRecord(token);
  return ref && { kind: ref.kind, code: ref.code, id: ref.id, board: ref.board };
};

const must = (token: string) => {
  const ref = parseRecord(token);
  if (!ref) throw new Error(`${token} is not a record`);
  return ref;
};

const kind = (id: string) => {
  const found = kindOf(id);
  if (!found) throw new Error(`no kind ${id}`);
  return found;
};

describe('I-85 P3 the code table comes from the registry', { tags: ['critical'] }, () => {
  it('reads the longest code first, so TP, DP, PR, BL, WT never fall back to a shorter one', () => {
    expect(read('TP-3')).toMatchObject({ kind: 'test-plan', code: 'TP-3', id: 'I-3' });
    expect(read('T-3')).toMatchObject({ kind: 'test-case', id: 'T-3' });
    expect(read('DP-4')).toMatchObject({ kind: 'dependency', code: 'DP-4', id: 'D-4' });
    expect(read('D-4')).toMatchObject({ kind: 'decision', id: 'D-4' });
    expect(read('PR-12')).toMatchObject({ kind: 'change', code: 'PR-12' });
    expect(read('P-5')).toMatchObject({ kind: 'followup' });
    expect(read('F-5')).toMatchObject({ kind: 'followup' });
  });

  it('reads the keyed kinds by their name, version or id', () => {
    expect(read('BL-web')).toMatchObject({ kind: 'build', id: 'web' });
    expect(read('WT-agent-1')).toMatchObject({ kind: 'worktree', id: 'agent-1' });
    expect(read('V-1.2.3')).toMatchObject({ kind: 'release', id: '1.2.3' });
  });

  it('keeps the board prefix, and refuses what is not a code', () => {
    expect(read('rotp/BL-web')).toMatchObject({ kind: 'build', board: 'rotp' });
    expect(read('rotp/I-105')).toMatchObject({ id: 'I-105', board: 'rotp' });
    expect(read('Z-1')).toBeNull();
    expect(read('T-5x')).toBeNull();
    expect(read('t-5')).toBeNull();
  });

  it('reads the stored item id behind a typed code', () => {
    expect(must('E-12').id).toBe('I-12');
    expect(must('S-13').collection).toBe('items');
    expect(must('Q-4').collection).toBe('entries');
  });
});

describe('I-85 P3 the address of an open record', { tags: ['critical'] }, () => {
  it('writes ?open and ?pane, keeps other parameters, and drops the pane with no record', () => {
    expect(recordSearch('', 'I-12')).toBe('?open=I-12');
    expect(recordSearch('', 'I-12', 'links')).toBe('?open=I-12&pane=links');
    expect(recordSearch('?x=1&open=I-1&pane=log', 'I-2')).toBe('?x=1&open=I-2');
    expect(recordSearch('?open=I-1&pane=log', null, 'log')).toBe('');
  });

  it('builds the href of a record on this board and on another', () => {
    expect(recordHref(must('I-12'), 'argus')).toBe('/p/argus?open=I-12');
    expect(recordHref(must('rotp/I-12'), 'argus')).toBe('/p/rotp?open=I-12');
    expect(recordHref(must('I-12'), null)).toBeNull();
  });

  it('reads the record an href opens, only on its own board', () => {
    expect(openOfHref('/p/argus?open=I-12&pane=links', 'argus')).toEqual({ code: 'I-12', tab: 'links' });
    expect(openOfHref('/p/argus?open=I-12', 'argus')).toEqual({ code: 'I-12' });
    expect(openOfHref('/p/rotp?open=I-12', 'argus')).toBeNull();
    expect(openOfHref('/p/argus#kanban', 'argus')).toBeNull();
  });

  it('still opens the old hashes: item, item with a tab, test, agent, to-do', () => {
    expect(legacyOpen('#I-12')).toEqual({ code: 'I-12', tab: null });
    expect(legacyOpen('#I-12/links')).toEqual({ code: 'I-12', tab: 'links' });
    expect(legacyOpen('#T-4')).toEqual({ code: 'T-4', tab: null });
    expect(legacyOpen('#A-9')).toEqual({ code: 'A-9', tab: null });
    expect(legacyOpen('#H-3')).toEqual({ code: 'H-3', tab: null });
    expect(legacyOpen('#kanban')).toBeNull();
    expect(legacyOpen('#tests')).toBeNull();
  });
});

describe('I-85 P3 what a record view shows', { tags: ['critical'] }, () => {
  it('shows the code of a kind the way the registry names it', () => {
    expect(shownCode(kind('dependency'), 'D-4')).toBe('DP-4');
    expect(shownCode(kind('build'), 'web')).toBe('BL-web');
    expect(shownCode(kind('release'), '1.2.3')).toBe('V-1.2.3');
    expect(shownCode(kind('task'), 'I-12')).toBe('I-12');
  });

  it('takes the assignee of a record from the field the registry names', () => {
    expect(assigneeOfRecord(kind('question'), {}, 'argus')).toBe('owner');
    expect(assigneeOfRecord(kind('release'), {}, 'argus')).toBe('lead:argus');
    expect(assigneeOfRecord(kind('dependency'), { from: { project: 'rotp' } }, 'argus')).toBe('lead:rotp');
    expect(assigneeOfRecord(kind('build'), { by: 'worker' }, 'argus')).toBe('worker');
    expect(assigneeOfRecord(kind('test-case'), { thread: 'worker' }, 'argus')).toBe('worker');
  });

  it('groups by the field the registry names; one row may sit in many groups, a row with none goes to none', () => {
    expect(groupKeysOf(kind('worktree'), { item: 'I-1' })).toEqual(['I-1']);
    expect(groupKeysOf(kind('worktree'), {})).toEqual([]);
    expect(groupKeysOf(kind('test-case'), { plans: ['TP-1', 'TP-2'] })).toEqual(['TP-1', 'TP-2']);
  });

  it('puts closed groups last and ranks the open ones', () => {
    const rows = [{ id: 'a', g: 'x' }, { id: 'b', g: 'y' }, { id: 'c', g: 'z' }, { id: 'd', g: '' }];
    const ranks = new Map([['x', { rank: 2, closed: false }], ['y', { rank: 1, closed: false }], ['z', { rank: 0, closed: true }]]);
    const info = (key: string) => ranks.get(key) ?? null;
    const grouped = groupRecords(rows, (row) => (row.g ? [row.g] : []), info);
    expect(grouped.groups.map((group) => group.key)).toEqual(['y', 'x']);
    expect(grouped.closed.map((group) => group.key)).toEqual(['z']);
    expect(grouped.none.map((row) => row.id)).toEqual(['d']);
  });
});

describe('I-85 P3 every kind has its menu', { tags: ['critical'] }, () => {
  it.each(REGISTRY_KINDS.map((entry) => entry.id))('%s has actions and a Copy id entry', (id) => {
    const defs = RECORD_ACTIONS[id];
    expect(defs?.length).toBeGreaterThan(0);
    expect(defs?.some((def) => def.id === 'copy')).toBe(true);
  });

  it('offers only the actions that have a handler, edit group first and soft group last', () => {
    const groups = recordMenuGroups('task', 'I-1', { open: () => undefined, copy: () => undefined, delete: () => undefined });
    expect(groups.map((group) => group.id)).toEqual(['edit', 'soft']);
    expect(groups[1]?.items.flatMap((entry) => ('id' in entry ? [entry.id] : []))).toEqual(['delete', 'copy']);
  });

  it('puts the middle groups between the edit and the other groups', () => {
    const middle = { id: 'move', items: [] };
    const groups = recordMenuGroups('task', 'I-1', { open: () => undefined, copy: () => undefined }, [middle]);
    expect(groups.map((group) => group.id)).toEqual(['edit', 'move', 'soft']);
  });
});
