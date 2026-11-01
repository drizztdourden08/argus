/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { metaDoc, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import type { Responder } from './mount-tab';
import { confirmGroup, findRow, rowButton, text } from './wt-dom';

cleanAfterEach();

const docs = () => [
  worktree('a', 'stale', { dirty: 2, unmerged: 4, unmergedSubjects: ['wip <b>login</b>', 'fix the form', 'more'], branch: 'feat/a' }),
  worktree('b', 'merged', { merged: true, unmerged: 0, branch: 'b' }),
];

const removed = (extra: object = {}): Responder => () => ({ status: 200, body: { removed: true, prunedOnly: false, ...extra } });

const confirmButton = (): HTMLButtonElement => within(confirmGroup('Confirm the removal')).getByRole<HTMLButtonElement>('button', { name: 'Confirm' });

describe('K3 Delete now: the confirm that says what is lost', { tags: ['standard'] }, () => {
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

describe('K3 Remove: the lighter confirm for a clean worktree', { tags: ['standard'] }, () => {
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
});

describe('K4 what the owner saw is what is sent', { tags: ['standard'] }, () => {
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
});
