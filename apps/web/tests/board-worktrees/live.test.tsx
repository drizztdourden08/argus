/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { metaDoc, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import { findRow, rowButton, rowNames, rowOf } from './wt-dom';

cleanAfterEach();

describe('S1 the tab follows the stream', { tags: ['standard'] }, () => {
  it('shows a new worktree, a changed status and a removed row as the changes arrive', async () => {
    const { change, drop } = await mountTab({ docs: [worktree('a', 'active', { ageDays: 0 })], meta: metaDoc() });
    await findRow('a');
    change('worktrees', 'id-b', worktree('b', 'stale'));
    await findRow('b');
    expect(rowNames()).toEqual(['b', 'a']);
    change('worktrees', 'id-a', worktree('a', 'merged', { merged: true, unmerged: 0, ageDays: 9 }));
    await waitFor(() => expect(within(rowOf('a')).getByTitle(/can go\. The thread was told/)).toBeTruthy());
    expect(rowNames()).toEqual(['a', 'b']);
    drop('worktrees', 'id-b');
    await waitFor(() => expect(rowNames()).toEqual(['a']));
  });

  it('keeps the open form on its row while other rows change', async () => {
    const { change } = await mountTab({ docs: [worktree('a', 'stale'), worktree('b', 'stale')], meta: metaDoc() });
    await findRow('a');
    fireEvent.click(rowButton('a', 'Ask the thread to delete'));
    change('worktrees', 'id-b', worktree('b', 'merged', { merged: true, unmerged: 0, ageDays: 9 }));
    await waitFor(() => expect(rowButton('b', 'Remove')).toBeTruthy());
    expect(screen.getByRole('group', { name: 'Ask lead to delete a' })).toBeTruthy();
  });
});
