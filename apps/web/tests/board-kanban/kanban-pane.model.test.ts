/* @layer renderer-app @kind test */
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { ITEM_REGISTRY, itemTypeOf } from '../../src/data/registry';
import { hashTab, itemHash, STATUS_OPTIONS } from '../../src/screens/board/tabs/kanban/model';
import { casesBadge } from '../../src/screens/board/tabs/kanban/pane/model/cases-badge';
import { childrenBadge } from '../../src/screens/board/tabs/kanban/pane/model/children-badge';
import { childTypes } from '../../src/screens/board/tabs/kanban/pane/model/child-types';
import { crumbsOf } from '../../src/screens/board/tabs/kanban/pane/model/crumbs-of';
import { discussionBadge } from '../../src/screens/board/tabs/kanban/pane/model/discussion-badge';
import { fieldGroups } from '../../src/screens/board/tabs/kanban/pane/model/field-groups';
import { fieldProblem } from '../../src/screens/board/tabs/kanban/pane/model/field-problem';
import { linksBadge } from '../../src/screens/board/tabs/kanban/pane/model/links-badge';
import { nodeOpens } from '../../src/screens/board/tabs/kanban/pane/model/node-opens';
import { parentChoices } from '../../src/screens/board/tabs/kanban/pane/model/parent-choices';
import { parseTrail } from '../../src/screens/board/tabs/kanban/pane/model/parse-trail';
import { stageFact } from '../../src/screens/board/tabs/kanban/pane/model/stage-fact';
import { tabsOf } from '../../src/screens/board/tabs/kanban/pane/model/tabs-of';
import { testsBadge } from '../../src/screens/board/tabs/kanban/pane/model/tests-badge';
import { textValue } from '../../src/screens/board/tabs/kanban/pane/model/text-value';
import type { PaneData, TrailNode } from '../../src/screens/board/tabs/kanban/pane/pane.type';
import { NOW, itemRef } from './kanban-docs';

const PANE = join(import.meta.dirname, '../../src/screens/board/tabs/kanban/pane');

const SOURCES = ['.', 'tabs', 'fields', 'trail'].flatMap((dir) =>
  readdirSync(join(PANE, dir))
    .filter((file) => file.endsWith('.tsx'))
    .map((file): [string, string] => [`${dir}/${file}`, readFileSync(join(PANE, dir, file), 'utf8')]),
);

const data = (extra: Partial<PaneData> = {}): PaneData => ({
  boardKey: 'alpha',
  item: itemRef('I-1'),
  items: [],
  tests: [],
  allTests: [],
  threads: [],
  record: { questions: [], decisions: [], notes: [], todos: [], timeline: [] },
  waiting: false,
  blocking: null,
  now: NOW,
  ops: {} as never,
  acts: {} as never,
  feed: {} as never,
  open: () => undefined,
  ...extra,
});

const node = (extra: Partial<TrailNode> = {}): TrailNode => ({
  project: 'alpha',
  item: 'I-2',
  title: 'Two',
  type: 'task',
  lane: 'doing',
  stage: 'doing',
  assignee: 'worker',
  edge: { kind: 'item', id: 'I-2', state: 'doing', since: null },
  cycle: false,
  children: [],
  ...extra,
});

describe('I-72 P6 the tabs of an item come from the registry', { tags: ['standard'] }, () => {
  it.each(ITEM_REGISTRY.types.map((type) => [type.id, type.tabs]))('gives a %s the tabs its registry entry lists, in order', (id, tabs) => {
    expect(tabsOf(itemRef('I-1', { type: id as string }))).toEqual(tabs);
  });

  it('reads an item without a type, or with an unknown one, as a task', () => {
    const task = itemTypeOf('task')?.tabs;
    expect(tabsOf(itemRef('I-1'))).toEqual(task);
    expect(tabsOf(itemRef('I-1', { type: 'nothing' }))).toEqual(task);
  });

  it('opens a tab from the hash after the item id, and leaves other hashes alone', () => {
    expect(hashTab('#I-136/links')).toBe('links');
    expect(hashTab('#I-136')).toBeNull();
    expect(itemHash('#I-136/links')).toBe('I-136');
    expect(itemHash('#I-136')).toBe('I-136');
    expect(itemHash('#kanban')).toBeNull();
    expect(hashTab('#kanban')).toBeNull();
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
    expect(childrenBadge(data({ items }))?.value).toBe(1);
    expect(childrenBadge(data())).toBeUndefined();
  });

  it('counts the cases of a plan', () => {
    expect(casesBadge(data({ item: itemRef('I-1', { fields: { cases: ['T-1', 'T-2'] } }) }))?.value).toBe(2);
    expect(casesBadge(data())).toBeUndefined();
  });

  it('counts the links that hold the item back, in warning', () => {
    const item = itemRef('I-1', { blocked: [{ kind: 'item', id: 'I-2', label: 'Two' }, { kind: 'ask', id: 'Q-1', label: 'Why' }] });
    expect(linksBadge(data({ item }))).toMatchObject({ value: 2, tone: 'warn' });
    expect(linksBadge(data())).toBeUndefined();
  });

  it('counts the open questions and the pending to-dos', () => {
    const record = {
      questions: [{ entry: { id: 'Q-1' }, open: true, replies: [] }, { entry: { id: 'Q-2' }, open: false, replies: [] }],
      decisions: [],
      notes: [],
      todos: [{ action: { id: 'H-1' }, done: false }, { action: { id: 'H-2' }, done: true }],
      timeline: [],
    } as never;
    expect(discussionBadge(data({ record }))?.value).toBe(2);
    expect(discussionBadge(data())).toBeUndefined();
  });

  it('shows the failing tests over the total, in danger while one fails', () => {
    const tests = [{ id: 'T-1', status: 'fail' }, { id: 'T-2', status: 'pass' }, { id: 'T-3', status: 'pass' }] as never;
    expect(testsBadge(data({ tests }))).toMatchObject({ value: '1/3', tone: 'danger' });
    expect(testsBadge(data({ tests: [{ id: 'T-2', status: 'pass' }] as never }))).toMatchObject({ value: '0/1', tone: 'neutral' });
    expect(testsBadge(data())).toBeUndefined();
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
    expect(fieldGroups(itemTypeOf('test-plan') as never).map((group) => group.fields)).toEqual([['bundle', 'env']]);
    expect(fieldGroups(itemTypeOf('task') as never)).toEqual([]);
  });

  it('lists the fields of a type that has no group under Details', () => {
    expect(fieldGroups(itemTypeOf('test-case') as never)).toEqual([{ id: 'more', label: 'Details', fields: ['setup', 'steps', 'pass', 'fail', 'mode', 'bundle'] }]);
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
    expect(fieldProblem('foundIn', 'bad id!', {})).toBeDefined();
    expect(fieldProblem('cases', 'T-1, T-2', {})).toBeUndefined();
    expect(fieldProblem('target', '2026-02-30', {})).toBeDefined();
    expect(fieldProblem('repro', 'A lone paragraph of words.', {})).toBeDefined();
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

describe('I-72 P6 the dependency trail', { tags: ['standard'] }, () => {
  it('reads the tree the server sends, with its depth, open count and nested nodes', () => {
    const raw = {
      depth: 3,
      open: 2,
      root: {
        project: 'bxel', item: 'I-136', title: 'Lobby', type: 'task', lane: 'doing', stage: 'doing', assignee: 'worker', edge: null, cycle: false,
        children: [{
          project: 'tessera', item: 'I-70', title: 'Seats', type: 'task', lane: 'doing', stage: null, assignee: 'lead:tessera',
          edge: { kind: 'dep-waits', id: 'D-4', state: 'open', since: '2026-10-25T12:00:00.000Z' }, cycle: false,
          children: [{ project: 'brock', item: 'Q-4', title: 'Which?', type: 'question', lane: 'open', stage: null, assignee: null, edge: { kind: 'ask', id: 'Q-4', state: 'open', since: null }, cycle: false, children: [] }],
        }],
      },
    };
    const tree = parseTrail(raw);
    expect(tree).toMatchObject({ depth: 3, open: 2 });
    expect(tree?.root.children[0]?.children[0]).toMatchObject({ item: 'Q-4', type: 'question', edge: { kind: 'ask', id: 'Q-4' } });
    expect(tree?.root.children[0]?.edge).toMatchObject({ kind: 'dep-waits', state: 'open' });
  });

  it('refuses what is not a trail', () => {
    expect(parseTrail(null)).toBeNull();
    expect(parseTrail({ id: 'entry', status: 'applied' })).toBeNull();
    expect(parseTrail({ depth: 1, root: { title: 'no item' } })).toBeNull();
  });

  it('opens an item of this board by its pane, one of another board through its board, an entry as an entry', () => {
    expect(nodeOpens(node({ edge: null }), 'alpha')).toBeNull();
    expect(nodeOpens(node(), 'alpha')).toEqual({ kind: 'item', id: 'I-2' });
    expect(nodeOpens(node({ project: 'rotp', item: 'I-12' }), 'alpha')).toEqual({ kind: 'board', href: '/p/rotp#I-12' });
    expect(nodeOpens(node({ item: 'Q-4', type: 'question' }), 'alpha')).toEqual({ kind: 'entry', id: 'Q-4' });
    expect(nodeOpens(node({ item: 'D-9', type: 'decision' }), 'alpha')).toEqual({ kind: 'entry', id: 'D-9' });
    expect(nodeOpens(node({ item: 'T-3', type: 'test' }), 'alpha')).toEqual({ kind: 'test', id: 'T-3' });
  });

  it('opens nothing for the leaves that have no pane', () => {
    expect(nodeOpens(node({ item: 'H-1', type: 'todo' }), 'alpha')).toBeNull();
    expect(nodeOpens(node({ item: 'D-4', type: 'request' }), 'alpha')).toBeNull();
    expect(nodeOpens(node({ item: 'I-9', type: 'legacy' }), 'alpha')).toBeNull();
  });
});
