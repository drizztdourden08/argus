/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { requiresParent } from '../../src/data/registry';
import { createReady } from '../../src/screens/board/tabs/kanban/model';

describe('I-166 an epic is required', { tags: ['critical'] }, () => {
  it('requires a parent for story, task, bug, debt and test plan, and not for an epic', () => {
    for (const type of ['story', 'task', 'bug', 'debt', 'test-plan']) expect(requiresParent(type)).toBe(true);
    expect(requiresParent('epic')).toBe(false);
  });

  it('keeps Create disabled until a thread and a required parent are picked', () => {
    expect(createReady('worker', true, '')).toBe(false);
    expect(createReady('worker', true, 'I-1')).toBe(true);
    expect(createReady('', true, 'I-1')).toBe(false);
    expect(createReady('worker', false, '')).toBe(true);
  });
});
