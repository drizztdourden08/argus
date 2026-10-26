/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ago, leftover, metaDoc, oldMeta, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import { cellOf, findRow, hasRowButton, rowButton, rowNames, rowOf, text } from './wt-dom';

cleanAfterEach();

describe('the tab follows the stream', () => {
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

  it('shows the answer of the thread as soon as it arrives', async () => {
    const ask = { id: 'O-1', kind: 'delete' as const, text: 'it can go', at: ago(60_000), by: 'owner', answer: null };
    const { change } = await mountTab({ docs: [worktree('a', 'stale', { request: ask })], meta: metaDoc() });
    await findRow('a');
    expect(text(cellOf('a', 'request'))).toContain('asked to delete, waiting');
    change('worktrees', 'id-a', worktree('a', 'stale', { request: { ...ask, answer: { note: 'kept for the login work', at: ago(1000), by: 'lead' } } }));
    await waitFor(() => expect(text(cellOf('a', 'request'))).toContain('kept for the login work'));
    expect(text(cellOf('a', 'request'))).not.toContain('waiting');
  });

  it('a row that turns into a folder being deleted loses its buttons and shows the job', async () => {
    const { change } = await mountTab({ docs: [leftover('a')], meta: oldMeta() });
    await findRow('a');
    expect(hasRowButton('a', 'Delete leftover')).toBe(true);
    change('worktrees', 'id-a', leftover('a', { status: 'deleting', job: { id: 'L-1', state: 'running', removed: 1, total: 4 } }));
    await waitFor(() => expect(hasRowButton('a', 'Delete leftover')).toBe(false));
    expect(text(cellOf('a', 'status'))).toContain('deleting, 1 of 4 entries');
  });

  it('keeps the open form on its row while other rows change', async () => {
    const { change } = await mountTab({ docs: [worktree('a', 'stale'), worktree('b', 'stale')], meta: metaDoc() });
    await findRow('a');
    fireEvent.click(rowButton('a', 'Ask the thread to delete'));
    change('worktrees', 'id-b', worktree('b', 'merged', { merged: true, unmerged: 0, ageDays: 9 }));
    await waitFor(() => expect(rowButton('b', 'Remove')).toBeTruthy());
    expect(screen.getByRole('group', { name: 'Ask lead to delete a' })).toBeTruthy();
  });

  it('updates the summary and the batch button when the summary doc changes', async () => {
    const { change } = await mountTab({ docs: [leftover('a'), leftover('b')], meta: oldMeta() });
    await findRow('a');
    expect(screen.getByRole('button', { name: /Delete all leftovers older than 3 days \(2,/ })).toBeTruthy();
    change('meta', 'worktrees', { ...oldMeta({ leftoverOld: { days: 3, count: 1, sizeMB: 512 } }) });
    await waitFor(() => expect(screen.getByRole('button', { name: 'Delete all leftovers older than 3 days (1, about 512 MB)' })).toBeTruthy());
  });
});
