/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { itemTypeOf } from '../../src/data/registry';
import type { Doc } from './kanban-harness';
import { DAY, MIN, ago, cleanKanban, item, mountKanban } from './kanban-harness';

cleanKanban();

const caseDoc = (extra: Doc = {}): Doc => ({
  id: 'T-1',
  item: 'I-1',
  thread: 'worker',
  title: 'Case T-1',
  mode: 'manual',
  setup: [{ label: 'profile', text: 'story-events' }],
  steps: ['open it', 'look at it'],
  pass: ['it opens'],
  fail: ['it breaks'],
  ac: [1],
  status: 'pending',
  result: null,
  runs: [],
  proofs: [],
  rank: 1,
  rev: 1,
  createdAt: ago(DAY),
  updatedAt: ago(MIN),
  ...extra,
});

const mountCase = async (extra: Doc = {}) => {
  await mountKanban({ items: [item('I-1', { ac: [{ text: 'The sage speaks', done: true }] })], tests: [caseDoc(extra)] }, undefined, '#T-1');
  return screen.findByRole('dialog', { name: 'Test detail' });
};

describe('I-85 R11 the test case overview is drawn from the registry', { tags: ['critical'] }, () => {
  it('names the groups the registry gives the test case, in its order', async () => {
    const groups = itemTypeOf('test-case')?.groups ?? [];
    expect(groups.map((group) => group.id)).toEqual(['setup', 'expected']);
    const dialog = await mountCase();
    for (const group of groups) expect(within(dialog).getByRole('region', { name: group.label })).toBeTruthy();
  });

  it('draws the lines of the steps and of the expected results', async () => {
    const dialog = await mountCase();
    expect(within(within(dialog).getByRole('list', { name: 'Steps' })).getAllByRole('listitem')).toHaveLength(2);
    const expected = within(dialog).getByRole('region', { name: 'Expected' });
    expect(within(expected).getByText('it opens')).toBeTruthy();
    expect(within(expected).getByText('it breaks')).toBeTruthy();
  });

  it('draws nothing for a group whose fields are all empty', async () => {
    const dialog = await mountCase({ setup: [], steps: [], pass: [], fail: [] });
    expect(within(dialog).queryByRole('region', { name: 'Setup' })).toBeNull();
    expect(within(dialog).queryByRole('region', { name: 'Expected' })).toBeNull();
  });
});
