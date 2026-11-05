/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { ITEM_REGISTRY, allowsParent, fieldOf, firstStageIn, itemTypeOf, knowsLead, laneOf, limitOf, requiresParent, typeHasField } from '../../src/data/registry';

const stageOf = (type: string, stage: string) => itemTypeOf(type)?.stages.find((entry) => entry.id === stage);

describe('K2 the web reads the item type registry file', { tags: ['critical'] }, () => {
  it('keeps the test runs off the kanban and the plan without bundle', () => {
    expect(itemTypeOf('test-run')?.hideOnKanban).toBe(true);
    expect(itemTypeOf('test-plan')?.hideOnKanban).toBeUndefined();
    expect(typeHasField('test-plan', 'bundle')).toBe(false);
    expect(typeHasField('test-run', 'results')).toBe(true);
  });

  it('lists the types of the design in display order and the five lanes', () => {
    expect(ITEM_REGISTRY.types.map((type) => type.id)).toEqual(['epic', 'story', 'task', 'bug', 'debt', 'test-plan', 'test-run', 'test-case', 'build', 'worktree']);
    expect(ITEM_REGISTRY.lanes).toEqual(['todo', 'doing', 'review', 'done', 'dropped']);
  });

  it('maps a stage to its lane and a lane to its first stage', () => {
    expect(laneOf('bug', 'fixing')).toBe('doing');
    expect(laneOf('bug', 'wontfix')).toBe('dropped');
    expect(laneOf('bug', 'nope')).toBeUndefined();
    expect(laneOf('ghost', 'fixing')).toBeUndefined();
    expect(firstStageIn('bug', 'todo')?.id).toBe('reported');
    expect(firstStageIn('bug', 'dropped')?.id).toBe('wontfix');
    expect(firstStageIn('story', 'todo')?.id).toBe('draft');
  });

  it('requires an epic parent for a story and a task only', () => {
    expect(requiresParent('story')).toBe(true);
    expect(requiresParent('task')).toBe(true);
    expect(['epic', 'bug', 'debt', 'test-plan', 'ghost'].some(requiresParent)).toBe(false);
  });

  it('follows the parent table of the design', () => {
    expect(allowsParent('story', 'epic')).toBe(true);
    expect(allowsParent('bug', 'story')).toBe(true);
    expect(allowsParent('story', 'story')).toBe(false);
    expect(allowsParent('epic', 'epic')).toBe(false);
    expect(allowsParent('ghost', 'epic')).toBe(false);
    expect(itemTypeOf('test-case')?.parents).toEqual([]);
  });

  it('gives a type its own fields and the base fields', () => {
    expect(typeHasField('bug', 'severity')).toBe(true);
    expect(typeHasField('bug', 'title')).toBe(true);
    expect(typeHasField('bug', 'goal')).toBe(false);
    expect(typeHasField('ghost', 'severity')).toBe(false);
  });

  it('keeps the bug flow moves and the test plan gates', () => {
    expect(stageOf('bug', 'verifying')?.next).toEqual(['closed', 'fixing']);
    expect((stageOf('test-plan', 'ready')?.gates ?? []).map((gate) => gate.need)).toEqual(['cases']);
    expect(stageOf('bug', 'duplicate')?.requires).toEqual(['duplicateOf']);
  });

  it('marks which types refuse or warn when they are done', () => {
    expect(itemTypeOf('story')?.done?.mode).toBe('refuse');
    expect(itemTypeOf('task')?.done?.mode).toBe('warn');
    expect(itemTypeOf('test-case')?.done).toBeUndefined();
  });

  it('looks up a field, a limit and a lead, and nothing that only the object prototype has', () => {
    expect(fieldOf('title')).toMatchObject({ content: 'line', max: 80, required: true });
    expect(fieldOf('constructor')).toBeUndefined();
    expect(limitOf('name')).toBe(40);
    expect(limitOf('toString')).toBeUndefined();
    expect(knowsLead('brock')).toBe(true);
    expect(knowsLead('nobody')).toBe(false);
    expect(knowsLead('constructor')).toBe(false);
  });
});
