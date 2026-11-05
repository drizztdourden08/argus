/* @layer renderer-app @kind test */
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { ITEM_REGISTRY, itemTypeOf } from '../../src/data/registry';
import { STATUS_OPTIONS } from '../../src/screens/board/tabs/kanban/model';
import { casesBadge } from '../../src/shell/records/pane/model/cases-badge';
import { childrenBadge } from '../../src/shell/records/pane/model/children-badge';
import { childTypes } from '../../src/shell/records/pane/model/child-types';
import { crumbsOf } from '../../src/shell/records/pane/model/crumbs-of';
import { discussionBadge } from '../../src/shell/records/pane/model/discussion-badge';
import { fieldGroups } from '../../src/shell/records/pane/model/field-groups';
import { fieldProblem } from '../../src/shell/records/pane/model/field-problem';
import { linksBadge } from '../../src/shell/records/pane/model/links-badge';
import { parentChoices } from '../../src/shell/records/pane/model/parent-choices';
import { stageFact } from '../../src/shell/records/pane/model/stage-fact';
import { tabsOf } from '../../src/shell/records/pane/model/tabs-of';
import { testsBadge } from '../../src/shell/records/pane/model/tests-badge';
import { textValue } from '../../src/shell/records/pane/model/text-value';
import { itemRef } from './kanban-docs';
import { paneData } from './pane-data';

const PANE = join(import.meta.dirname, '../../src/shell/records/pane');

const SOURCES = ['.', 'tabs', 'fields', 'trail'].flatMap((dir) =>
  readdirSync(join(PANE, dir))
    .filter((file) => file.endsWith('.tsx'))
    .map((file): [string, string] => [`${dir}/${file}`, readFileSync(join(PANE, dir, file), 'utf8')]),
);

describe('I-72 P6 the tabs of an item come from the registry', { tags: ['standard'] }, () => {
  it.each(ITEM_REGISTRY.types.map((type) => [type.id, type.tabs]))('gives a %s the tabs its registry entry lists, in order', (id, tabs) => {
    expect(tabsOf(itemRef('I-1', { type: id }))).toEqual(tabs);
  });

  it('reads an item without a type, or with an unknown one, as a task', () => {
    const task = itemTypeOf('task')?.tabs;
    expect(tabsOf(itemRef('I-1'))).toEqual(task);
    expect(tabsOf(itemRef('I-1', { type: 'nothing' }))).toEqual(task);
  });
});

describe('I-72 P6 no component of the pane names a type', { tags: ['standard'] }, () => {
  it.each(SOURCES)('%s names no item type', (_file, source) => {
    for (const type of ITEM_REGISTRY.types) expect(source).not.toMatch(new RegExp(`['"\`]${type.id}['"\`]`));
  });
});

describe('I-72 P6 the badges of the tabs', { tags: ['standard'] }, () => {
  it('counts the open children, never the done, dropped or archived ones', () => {
    const items = [
      itemRef('I-2', { parent: 'I-1' }),
      itemRef('I-3', { parent: 'I-1', status: 'done' }),
      itemRef('I-4', { parent: 'I-1', status: 'dropped' }),
      itemRef('I-5', { parent: 'I-1', archivedAt: '2026-10-01T00:00:00.000Z' }),
      itemRef('I-6', { parent: 'I-9' }),
    ];
    expect(childrenBadge(paneData({ items }))?.value).toBe(1);
    expect(childrenBadge(paneData())).toBeUndefined();
  });

  it('counts the cases of a plan', () => {
    expect(casesBadge(paneData({ item: itemRef('I-1', { fields: { cases: ['T-1', 'T-2'] } }) }))?.value).toBe(2);
    expect(casesBadge(paneData())).toBeUndefined();
  });

  it('counts the links that hold the item back, in warning', () => {
    const item = itemRef('I-1', { blocked: [{ kind: 'item', id: 'I-2', label: 'Two' }, { kind: 'ask', id: 'Q-1', label: 'Why' }] });
    expect(linksBadge(paneData({ item }))).toMatchObject({ value: 2, tone: 'warn' });
    expect(linksBadge(paneData())).toBeUndefined();
  });

  it('counts the open questions and the pending to-dos', () => {
    const record = {
      questions: [{ entry: { id: 'Q-1' }, open: true, replies: [] }, { entry: { id: 'Q-2' }, open: false, replies: [] }],
      decisions: [],
      notes: [],
      todos: [{ action: { id: 'H-1' }, done: false }, { action: { id: 'H-2' }, done: true }],
      timeline: [],
    } as never;
    expect(discussionBadge(paneData({ record }))?.value).toBe(2);
    expect(discussionBadge(paneData())).toBeUndefined();
  });

  it('shows the number of failing tests, in danger while one fails', () => {
    const tests = [{ id: 'T-1', status: 'fail' }, { id: 'T-2', status: 'pass' }, { id: 'T-3', status: 'pass' }] as never;
    expect(testsBadge(paneData({ tests }))).toMatchObject({ value: 1, tone: 'danger', title: '1 failing of 3 tests' });
    expect(testsBadge(paneData({ tests: [{ id: 'T-2', status: 'pass' }] as never }))).toMatchObject({ value: 0, tone: 'neutral' });
    expect(testsBadge(paneData())).toBeUndefined();
  });
});

describe('I-72 P6 the fields of a type', { tags: ['standard'] }, () => {
  it('groups a bug as the registry does, and puts the rest under More', () => {
    const groups = fieldGroups(itemTypeOf('bug') as never);
    expect(groups.map((group) => [group.label, group.fields])).toEqual([
      ['Report', ['severity', 'environment', 'foundIn', 'reporter']],
      ['Steps', ['repro', 'expected', 'actual']],
      ['More', ['duplicateOf', 'reason']],
    ]);
  });

  it('leaves the cases and the runs of a plan to their own tabs, and the base fields to the Overview', () => {
    expect(fieldGroups(itemTypeOf('test-plan') as never).map((group) => group.fields)).toEqual([['env']]);
    expect(fieldGroups(itemTypeOf('task') as never)).toEqual([]);
  });

  it('lists the fields of a type that has no group under Details', () => {
    expect(fieldGroups(itemTypeOf('build') as never)).toEqual([{ id: 'more', label: 'Details', fields: ['for', 'master', 'cmd', 'cwd', 'kind', 'branch', 'commit', 'window', 'current', 'bundle', 'useEvery'] }]);
  });

  it('turns the text of a field into its value: a list per line, ids split on commas', () => {
    expect(textValue('steps', 'open it\n\n  look  ')).toEqual(['open it', 'look']);
    expect(textValue('cases', 'T-1, T-2 T-3')).toEqual(['T-1', 'T-2', 'T-3']);
    expect(textValue('environment', '  bundle-16 ')).toBe('bundle-16');
  });

  it('judges the text of a field with the shared content rules', () => {
    expect(fieldProblem('environment', 'bundle-16', {})).toBeUndefined();
    expect(fieldProblem('environment', 'x'.repeat(121), {})).toBeDefined();
    expect(fieldProblem('severity', 'S9', {})).toBeDefined();
    expect(fieldProblem('severity', 'S2', {})).toBeUndefined();
    expect(fieldProblem('severity', '', {})).toBeDefined();
    expect(fieldProblem('foundIn', 'T-57', {})).toBeUndefined();
    expect(fieldProblem('foundIn', 'bad id', {})).toBeDefined();
    expect(fieldProblem('cases', 'T-1, T-2', {})).toBeUndefined();
    expect(fieldProblem('target', '2026-02-30', {})).toBeDefined();
    expect(fieldProblem('repro', 'word '.repeat(30), {})).toBeDefined();
    expect(fieldProblem('repro', '- open the lobby\n- look', {})).toBeUndefined();
  });
});

describe('I-72 P6 the facts of an item', { tags: ['standard'] }, () => {
  it('keeps the status select of an item without a type', () => {
    const fact = stageFact(itemRef('I-1', { status: 'doing' }));
    expect(fact).toMatchObject({ label: 'Status', value: 'doing', options: STATUS_OPTIONS });
    expect(fact.target('review')).toEqual({ status: 'review' });
  });

  it('offers a typed item its stage and the stages that follow, and moves it by stage', () => {
    const fact = stageFact(itemRef('I-1', { type: 'bug', stage: 'reported' }));
    expect(fact.label).toBe('Stage');
    expect(fact.value).toBe('reported');
    expect(fact.options.map((option) => option.value)).toEqual(['reported', 'confirmed', 'wontfix', 'duplicate']);
    expect(fact.target('confirmed')).toEqual({ stage: 'confirmed' });
  });

  it('reads the stage of a typed item that has none from its lane', () => {
    expect(stageFact(itemRef('I-1', { type: 'story', status: 'doing' })).value).toBe('building');
  });

  it('limits the parent choices to the allowed types, never the item or what is below it', () => {
    const epic = itemRef('I-1', { type: 'epic' });
    const story = itemRef('I-2', { type: 'story', parent: 'I-1' });
    const task = itemRef('I-3', { type: 'task', parent: 'I-2' });
    const other = itemRef('I-4', { type: 'bug' });
    const items = [epic, story, task, other];
    expect(parentChoices(task, items).map((option) => option.value)).toEqual(['no-parent', 'I-1', 'I-2']);
    expect(parentChoices(story, items).map((option) => option.value)).toEqual(['no-parent', 'I-1']);
    expect(parentChoices(epic, items)).toEqual([]);
  });

  it('lists the types a child can have, from the registry', () => {
    expect(childTypes(itemRef('I-1', { type: 'epic' })).map((type) => type.id)).toEqual(['story', 'task', 'bug', 'debt', 'test-plan']);
    expect(childTypes(itemRef('I-1', { type: 'task' }))).toEqual([]);
  });

  it('walks up the parents for the crumbs, root first, and stops on a loop', () => {
    const epic = itemRef('I-1', { type: 'epic' });
    const story = itemRef('I-2', { type: 'story', parent: 'I-1' });
    const task = itemRef('I-3', { parent: 'I-2' });
    expect(crumbsOf(task, [epic, story, task]).map((crumb) => crumb.id)).toEqual(['I-1', 'I-2']);
    const a = itemRef('I-5', { parent: 'I-6' });
    const b = itemRef('I-6', { parent: 'I-5' });
    expect(crumbsOf(a, [a, b]).map((crumb) => crumb.id)).toEqual(['I-6']);
  });
});
