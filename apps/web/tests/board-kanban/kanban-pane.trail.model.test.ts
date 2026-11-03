/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { nodeOpens } from '../../src/screens/board/tabs/kanban/pane/model/node-opens';
import { parseTrail } from '../../src/screens/board/tabs/kanban/pane/model/parse-trail';
import type { TrailNode } from '../../src/screens/board/tabs/kanban/pane/pane.type';

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
