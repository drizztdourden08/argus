/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { WithId } from '../../src/data/react';
import type { Test } from '../../src/data/types';
import { branchKeys } from '../../src/screens/board/tabs/kanban-tree/model/branch-keys';
import { buildTree } from '../../src/screens/board/tabs/kanban-tree/model/build-tree';
import { closedAfter } from '../../src/screens/board/tabs/kanban-tree/model/closed-after';
import { indexNodes } from '../../src/screens/board/tabs/kanban-tree/model/index-nodes';
import { openKeys } from '../../src/screens/board/tabs/kanban-tree/model/open-keys';
import { progressOf } from '../../src/screens/board/tabs/kanban-tree/model/progress-of';
import { pruneTree } from '../../src/screens/board/tabs/kanban-tree/model/prune-tree';
import type { TreeNode, TreeSource } from '../../src/screens/board/tabs/kanban-tree/tree.type';
import { HOUR, before, itemRef } from './kanban-docs';

const testRef = (id: string, extra: Partial<Test> = {}): WithId<Test> =>
  ({ id, title: `Case ${id}`, status: 'pending', mode: 'manual', thread: null, item: null, ...extra }) as WithId<Test>;

const sourceOf = (items: ReturnType<typeof itemRef>[], tests: WithId<Test>[] = [], archived = false): TreeSource => ({ items, tests, archived });

const idsOf = (nodes: readonly TreeNode[]): string[] => nodes.map((node) => node.key);

const find = (nodes: readonly TreeNode[], key: string): TreeNode => {
  const node = indexNodes(nodes).get(key);
  if (!node) throw new Error(`no node ${key}`);
  return node;
};

const HIERARCHY = [
  itemRef('I-1', { type: 'epic', rank: 1 }),
  itemRef('I-2', { type: 'story', parent: 'I-1', rank: 2 }),
  itemRef('I-3', { type: 'task', parent: 'I-2', rank: 1, status: 'done' }),
  itemRef('I-4', { type: 'bug', parent: 'I-2', rank: 2 }),
  itemRef('I-5', { type: 'task', parent: 'I-1', rank: 1 }),
];

describe('I-70 P9 the tree nests items by their parent', { tags: ['critical'] }, () => {
  it('nests epic, story and leaf, children in rank order', () => {
    const roots = buildTree(sourceOf(HIERARCHY));
    expect(idsOf(roots)).toEqual(['I-1']);
    expect(idsOf(find(roots, 'I-1').children)).toEqual(['I-5', 'I-2']);
    expect(idsOf(find(roots, 'I-2').children)).toEqual(['I-3', 'I-4']);
  });

  it('puts an item without a parent at the top level, in rank order', () => {
    const roots = buildTree(sourceOf([itemRef('I-9', { rank: 9 }), ...HIERARCHY, itemRef('I-8', { rank: 0 })]));
    expect(idsOf(roots)).toEqual(['I-8', 'I-1', 'I-9']);
  });

  it('puts an orphan at the top level when its parent is deleted, archived or missing', () => {
    const items = [
      itemRef('I-1', { type: 'epic', deletedAt: before(HOUR) }),
      itemRef('I-2', { type: 'epic', archivedAt: before(HOUR) }),
      itemRef('I-3', { type: 'task', parent: 'I-1', rank: 1 }),
      itemRef('I-4', { type: 'task', parent: 'I-2', rank: 2 }),
      itemRef('I-5', { type: 'task', parent: 'I-404', rank: 3 }),
    ];
    expect(idsOf(buildTree(sourceOf(items)))).toEqual(['I-3', 'I-4', 'I-5']);
    const withArchived = buildTree(sourceOf(items, [], true));
    expect(idsOf(withArchived)).toEqual(['I-3', 'I-5', 'I-2']);
    expect(idsOf(find(withArchived, 'I-2').children)).toEqual(['I-4']);
  });

  it('puts an item at the top level when the registry does not allow that parent type', () => {
    const items = [itemRef('I-1', { type: 'task' }), itemRef('I-2', { type: 'task', parent: 'I-1' }), itemRef('I-3', { type: 'epic', parent: 'I-1' })];
    expect(idsOf(buildTree(sourceOf(items)))).toEqual(['I-1', 'I-2', 'I-3']);
  });

  it('reads an item with no type as a task', () => {
    const roots = buildTree(sourceOf([itemRef('I-1', { type: 'story' }), itemRef('I-2', { parent: 'I-1' })]));
    expect(find(roots, 'I-2').typeId).toBe('task');
    expect(find(roots, 'I-1').children).toHaveLength(1);
  });

  it('puts the cases of a test plan under it, in the order of its list, and skips unknown cases', () => {
    const plan = itemRef('I-1', { type: 'test-plan', fields: { cases: ['T-2', 'T-1', 'T-9'] } });
    const roots = buildTree(sourceOf([plan], [testRef('T-1'), testRef('T-2', { status: 'pass' })]));
    const cases = find(roots, 'I-1').children;
    expect(cases.map((node) => node.id)).toEqual(['T-2', 'T-1']);
    expect(cases.map((node) => node.kind)).toEqual(['case', 'case']);
    expect(cases.every((node) => node.open === 'TP-1')).toBe(true);
  });

  it('gives a manual case to the owner and an auto case to its thread', () => {
    const plan = itemRef('I-1', { type: 'test-plan', fields: { cases: ['T-1', 'T-2'] } });
    const roots = buildTree(sourceOf([plan], [testRef('T-1'), testRef('T-2', { mode: 'auto', thread: 'worker' })]));
    expect(find(roots, 'I-1/T-1').assignee).toBe('owner');
    expect(find(roots, 'I-1/T-2').assignee).toBe('worker');
  });
});

describe('I-70 P9 the progress of a node comes from its children', { tags: ['critical'] }, () => {
  it('counts the children done over the children, leaving out the dropped ones', () => {
    const items = [
      itemRef('I-1', { type: 'story' }),
      itemRef('I-2', { parent: 'I-1', status: 'done' }),
      itemRef('I-3', { parent: 'I-1' }),
      itemRef('I-4', { parent: 'I-1', status: 'dropped' }),
    ];
    const node = find(buildTree(sourceOf(items)), 'I-1');
    expect(node.count).toEqual({ done: 1, total: 2 });
    expect(progressOf(node)).toEqual({ text: '1/2', title: '1 of 2 children done' });
  });

  it('counts a test plan by its cases passed', () => {
    const plan = itemRef('I-1', { type: 'test-plan', fields: { cases: ['T-1', 'T-2', 'T-3'] } });
    const tests = [testRef('T-1', { status: 'pass' }), testRef('T-2', { status: 'fail' }), testRef('T-3')];
    const node = find(buildTree(sourceOf([plan], tests)), 'I-1');
    expect(progressOf(node)).toEqual({ text: '1/3', title: '1 of 3 cases passed' });
  });

  it('shows no progress on a leaf', () => {
    expect(progressOf(find(buildTree(sourceOf(HIERARCHY)), 'I-5'))).toBeNull();
  });

  it('keeps the true counts when the tree is pruned', () => {
    const roots = pruneTree(buildTree(sourceOf(HIERARCHY)), new Set(['I-4']));
    expect(find(roots, 'I-2').count).toEqual({ done: 1, total: 2 });
  });
});

describe('I-70 P9 the shown items prune the tree', { tags: ['critical'] }, () => {
  it('keeps the folders above a shown item and drops the branches with none', () => {
    const roots = pruneTree(buildTree(sourceOf(HIERARCHY)), new Set(['I-4']));
    expect(idsOf(roots)).toEqual(['I-1']);
    expect(idsOf(find(roots, 'I-1').children)).toEqual(['I-2']);
    expect(idsOf(find(roots, 'I-2').children)).toEqual(['I-4']);
  });

  it('keeps the cases of a shown plan and none for a plan that only holds the way', () => {
    const plan = itemRef('I-1', { type: 'test-plan', fields: { cases: ['T-1'] } });
    const tree = buildTree(sourceOf([plan], [testRef('T-1')]));
    expect(idsOf(find(pruneTree(tree, new Set(['I-1'])), 'I-1').children)).toEqual(['I-1/T-1']);
    expect(pruneTree(tree, new Set())).toEqual([]);
  });
});

describe('I-70 P9 folding', { tags: ['critical'] }, () => {
  it('lists the nodes that have children as the branches', () => {
    expect(branchKeys(buildTree(sourceOf(HIERARCHY)))).toEqual(['I-1', 'I-2']);
    expect(branchKeys(buildTree(sourceOf([itemRef('I-1')])))).toEqual([]);
  });

  it('opens every branch but the closed ones, and remembers what the tree closes', () => {
    const branches = ['I-1', 'I-2', 'I-6'];
    expect(openKeys(branches, new Set())).toEqual(branches);
    expect(openKeys(branches, new Set(['I-2']))).toEqual(['I-1', 'I-6']);
    expect([...closedAfter(branches, ['I-1', 'I-6'])]).toEqual(['I-2']);
    expect([...closedAfter(branches, [])]).toEqual(branches);
  });

  it('opens a branch that appears after the fold, since only the closed ones are kept', () => {
    const closed = closedAfter(['I-1', 'I-2'], ['I-1']);
    expect(openKeys(['I-1', 'I-2', 'I-7'], closed)).toEqual(['I-1', 'I-7']);
  });
});
