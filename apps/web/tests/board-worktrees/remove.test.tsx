/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ago, metaDoc, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import type { Responder } from './mount-tab';
import { confirmGroup, findRow, postedBodies, rowButton, text } from './wt-dom';

cleanAfterEach();

const REMOVE = '/worktrees/remove';

const docs = () => [
  worktree('a', 'stale', { dirty: 2, unmerged: 4, unmergedSubjects: ['wip <b>login</b>', 'fix the form', 'more'], branch: 'feat/a' }),
  worktree('b', 'merged', { merged: true, unmerged: 0, branch: 'b' }),
  worktree('c', 'stale', { detached: true, branch: null, unmerged: null }),
];

const removed = (extra: object = {}): Responder => () => ({ status: 200, body: { removed: true, prunedOnly: false, ...extra } });

const confirmButton = (): HTMLButtonElement => within(confirmGroup('Confirm the removal')).getByRole<HTMLButtonElement>('button', { name: 'Confirm' });

describe('Delete now: the confirm that says what is lost', () => {
  it('opens under the row with the uncommitted files, the unmerged commits with their first subjects, the branch, and the backup sentence', async () => {
    const { calls } = await mountTab({ docs: docs(), meta: metaDoc() });
    await findRow('a');
    fireEvent.click(rowButton('a', 'Delete now'));
    const group = confirmGroup('Confirm the removal');
    const words = text(group);
    expect(words).toContain('Delete X:/proj/trees/a now? This discards:');
    expect(words).toContain('2 uncommitted files (changed or new)');
    expect(words).toContain('the branch feat/a with its 4 commits not merged into main: "wip <b>login</b>", "fix the form", "more" and 1 more, if you also delete it');
    expect(words).toContain('A backup bundle is saved first (with a patch of the uncommitted changes)');
    expect(within(group).getByRole('checkbox', { name: 'also delete the branch feat/a' })).toBeTruthy();
    expect(rowButton('a', 'Delete now').getAttribute('aria-expanded')).toBe('true');
    expect(calls).toEqual([]);
  });

  it('a detached HEAD no branch keeps says its commits are lost and offers no branch to delete', async () => {
    await mountTab({ docs: docs(), meta: metaDoc() });
    await findRow('c');
    fireEvent.click(rowButton('c', 'Delete now'));
    const group = confirmGroup('Confirm the removal');
    expect(text(group)).toContain('commits not known to be in the default branch (on a detached HEAD that no branch keeps)');
    expect(within(group).queryByRole('checkbox')).toBeNull();
  });

  it('Confirm posts the removal with the discard, the branch when ticked, and the HEAD and dirty count seen; the answer shows the backup with its restore command', async () => {
    const reply = removed({
      branch: 'feat/a',
      deletedBranch: true,
      backup: { dir: 'X:/home/backups', bundle: 'X:/home/backups/a.bundle', patch: 'X:/home/backups/a.patch', restore: ['git -C "X:/proj" fetch "X:/home/backups/a.bundle" feat/a:feat/a'] },
    });
    const { calls } = await mountTab({ docs: docs(), meta: metaDoc() }, reply);
    await findRow('a');
    fireEvent.click(rowButton('a', 'Delete now'));
    fireEvent.click(within(confirmGroup('Confirm the removal')).getByRole('checkbox', { name: 'also delete the branch feat/a' }));
    fireEvent.click(confirmButton());
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({ url: '/api/p/alpha/worktrees/remove', method: 'POST' });
    expect(calls[0]?.body).toEqual({ id: 'id-a', branch: true, discard: true, head: 'abc1234', dirty: 2 });
    await waitFor(() => expect(screen.queryByRole('group', { name: 'Confirm the removal' })).toBeNull());
    const notice = screen.getByText('Removed a, branch feat/a deleted. Backup:').closest('[role="note"]');
    expect(notice).not.toBeNull();
    expect(text(notice as HTMLElement)).toContain('X:/home/backups/a.bundle');
    expect(text(notice as HTMLElement)).toContain('git -C "X:/proj" fetch "X:/home/backups/a.bundle" feat/a:feat/a');
    fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
    expect(screen.queryByText('Removed a, branch feat/a deleted. Backup:')).toBeNull();
  });

  it('does not delete anything without the click on Confirm: opening and cancelling post nothing', async () => {
    const { calls } = await mountTab({ docs: docs(), meta: metaDoc() });
    await findRow('a');
    fireEvent.click(rowButton('a', 'Delete now'));
    fireEvent.click(within(confirmGroup('Confirm the removal')).getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByRole('group', { name: 'Confirm the removal' })).toBeNull();
    fireEvent.click(rowButton('a', 'Delete now'));
    fireEvent.keyDown(confirmGroup('Confirm the removal'), { key: 'Escape' });
    expect(screen.queryByRole('group', { name: 'Confirm the removal' })).toBeNull();
    expect(calls).toEqual([]);
  });
});

describe('Remove: the lighter confirm for a clean worktree', () => {
  it('says nothing is lost and offers the branch', async () => {
    await mountTab({ docs: docs(), meta: metaDoc() });
    await findRow('b');
    fireEvent.click(rowButton('b', 'Remove'));
    const group = confirmGroup('Confirm the removal');
    expect(text(group)).toContain('Remove X:/proj/trees/b now? It is clean and its commits stay in the branch b: nothing is lost. The branch stays unless you tick the box and it is merged.');
    expect(text(group)).not.toContain('This discards');
    expect(text(group)).not.toContain('backup bundle');
    expect(within(group).getByRole('checkbox', { name: 'also delete the branch b' })).toBeTruthy();
  });

  it('Confirm posts no discard and keeps the branch unless ticked; the answer says the branch stayed', async () => {
    const { calls } = await mountTab({ docs: docs(), meta: metaDoc() }, removed({ branch: 'b', deletedBranch: false, branchNote: 'not merged' }));
    await findRow('b');
    fireEvent.click(rowButton('b', 'Remove'));
    fireEvent.click(confirmButton());
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]?.body).toEqual({ id: 'id-b', branch: false, discard: false, head: 'abc1234', dirty: 0 });
    expect(await screen.findByText('Removed b, branch b kept (not merged).')).toBeTruthy();
  });

  it('a refusal stays in the confirm with the sentence of the server, and it can be tried again', async () => {
    const refuse: Responder = () => ({ status: 409, body: { error: { code: 'conflict', message: 'X:/proj/trees/b changed since the listing <now>' } } });
    await mountTab({ docs: docs(), meta: metaDoc() }, refuse);
    await findRow('b');
    fireEvent.click(rowButton('b', 'Remove'));
    fireEvent.click(confirmButton());
    await waitFor(() => expect(text(confirmGroup('Confirm the removal'))).toContain('Not removed: X:/proj/trees/b changed since the listing <now>'));
    expect(confirmButton().disabled).toBe(false);
    fireEvent.click(within(confirmGroup('Confirm the removal')).getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByRole('group', { name: 'Confirm the removal' })).toBeNull();
  });

  it('the request form replaces the confirm on the same row', async () => {
    const { calls } = await mountTab({ docs: docs(), meta: metaDoc() });
    await findRow('b');
    fireEvent.click(rowButton('b', 'Remove'));
    fireEvent.click(rowButton('b', 'Ask the thread to delete'));
    expect(screen.queryByRole('group', { name: 'Confirm the removal' })).toBeNull();
    expect(screen.getByRole('group', { name: 'Ask lead to delete b' })).toBeTruthy();
    expect(calls).toEqual([]);
  });
});

describe('what the owner saw is what is sent', () => {
  it('a worktree that became dirty after the confirm opened blocks Confirm and says so', async () => {
    const { calls, change } = await mountTab({ docs: docs(), meta: metaDoc() });
    await findRow('b');
    fireEvent.click(rowButton('b', 'Remove'));
    expect(confirmButton().disabled).toBe(false);
    change('worktrees', 'id-b', { ...worktree('b', 'merged', { merged: true, unmerged: 0, branch: 'b', dirty: 2 }) });
    const group = confirmGroup('Confirm the removal');
    expect(text(group)).toContain('This changed since you opened this confirm.');
    expect(text(group)).toContain('This discards:');
    expect(confirmButton().disabled).toBe(true);
    fireEvent.click(confirmButton());
    expect(calls).toEqual([]);
  });

  it('a new HEAD blocks Confirm too, and reopening the confirm reads the new state', async () => {
    const { calls, change } = await mountTab({ docs: docs(), meta: metaDoc() });
    await findRow('b');
    fireEvent.click(rowButton('b', 'Remove'));
    change('worktrees', 'id-b', { ...worktree('b', 'merged', { merged: true, unmerged: 0, branch: 'b', head: 'fff9999' }) });
    expect(confirmButton().disabled).toBe(true);
    fireEvent.click(rowButton('b', 'Remove'));
    fireEvent.click(rowButton('b', 'Remove'));
    expect(confirmButton().disabled).toBe(false);
    expect(calls).toEqual([]);
  });

  it('a row that can no longer be removed blocks Confirm', async () => {
    const { change } = await mountTab({ docs: docs(), meta: metaDoc() });
    await findRow('b');
    fireEvent.click(rowButton('b', 'Remove'));
    change('worktrees', 'id-b', { ...worktree('b', 'merged', { merged: true, unmerged: 0, branch: 'b', locked: 'demo' }) });
    expect(confirmButton().disabled).toBe(true);
  });

  it('sends the HEAD and the dirty count of the moment the confirm opened', async () => {
    const { calls } = await mountTab({ docs: [worktree('d', 'stale', { dirty: 1, head: 'beef123', lastTouchedAt: ago(9 * 86_400_000) })], meta: metaDoc() }, removed());
    await findRow('d');
    fireEvent.click(rowButton('d', 'Delete now'));
    fireEvent.click(confirmButton());
    await waitFor(() => expect(postedBodies(calls, REMOVE)).toHaveLength(1));
    expect(postedBodies(calls, REMOVE)[0]).toEqual({ id: 'id-d', branch: false, discard: true, head: 'beef123', dirty: 1 });
  });
});
