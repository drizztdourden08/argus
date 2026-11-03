/* @layer renderer-app @kind test */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { itemRef } from '../board-kanban/kanban-docs';
import { groupOf, order } from '../helpers/group-dom';
import { metaDoc, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';

cleanAfterEach();

const board = () => ({
  meta: metaDoc(),
  items: [itemRef('I-1'), itemRef('I-2', { status: 'done' })],
  docs: [
    worktree('linked', 'active', { item: 'I-1' }),
    worktree('loose', 'active', { unlinked: true }),
    worktree('main-1', 'active', { main: true }),
  ],
});

describe('P6 the Worktrees tab by item', { tags: ['standard'] }, () => {
  it('groups the worktrees under their open item, then the ones with no item, then the main checkouts', async () => {
    const { container } = await mountTab(board());
    await screen.findByText('Main checkouts');
    expect(order(container)).toEqual(['I-1', 'none', 'main']);
    expect(groupOf(container, 'I-1').querySelector('[data-worktree="id-linked"]')).not.toBeNull();
    const none = groupOf(container, 'none');
    expect(within(none).getByText('No item')).toBeTruthy();
    expect(none.querySelector('[data-worktree="id-loose"]')).not.toBeNull();
    expect(groupOf(container, 'main').querySelector('[data-worktree="id-main-1"]')).not.toBeNull();
  });

  it('keeps the worktrees of a closed item in a Closed section', async () => {
    const { container } = await mountTab({ ...board(), docs: [worktree('old', 'stale', { item: 'I-2' })] });
    expect(await screen.findByText(/^Closed \(1\)/)).toBeTruthy();
    expect(groupOf(container, 'closed')).toBeTruthy();
  });
});
