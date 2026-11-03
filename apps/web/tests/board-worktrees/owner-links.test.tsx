/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { metaDoc, worktree } from './fixtures';
import { cleanAfterEach, mountTab } from './mount-tab';
import type { Responder } from './mount-tab';
import { chooseAction, hasAction, openMenu } from './wt-dom';

cleanAfterEach();

const item = (id: string, extra: object = {}) => ({ id, title: `Title of ${id}`, status: 'doing', thread: 'lead', type: 'task', ...extra });

const board = () => ({
  docs: [worktree('a', 'stale'), worktree('b', 'stale', { item: 'I-3', linkSource: 'manual' }), worktree('c', 'stale', { item: 'I-3', linkSource: 'branch' })],
  items: [item('I-3'), item('I-9', { status: 'done' }), item('I-8', { deletedAt: '2026-10-01T00:00:00.000Z' })],
  meta: metaDoc(),
});

const refused: Responder = () => ({ status: 200, body: { id: 'O-1', status: 'refused', outcome: 'no item "I-3" on this board', entry: null, seq: 1 } });

const dialog = (): Promise<HTMLElement> => screen.findByRole('dialog');

describe('I-85 Link a worktree to an item from the row menu', { tags: ['standard'] }, () => {
  it('offers the open items only and posts worktree-link with the chosen one, then closes', async () => {
    const { calls } = await mountTab(board());
    await chooseAction('a', /^Link to an item/);
    const box = await dialog();
    const link = within(box).getByRole<HTMLButtonElement>('button', { name: 'Link' });
    expect(link.disabled).toBe(true);
    fireEvent.click(within(box).getByRole('combobox', { name: 'Item' }));
    expect(screen.queryByRole('option', { name: /^I-9/ })).toBeNull();
    expect(screen.queryByRole('option', { name: /^I-8/ })).toBeNull();
    fireEvent.click(await screen.findByRole('option', { name: /^I-3/ }));
    expect(link.disabled).toBe(false);
    fireEvent.click(link);
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({ url: '/api/p/alpha/entries', method: 'POST' });
    expect(calls[0]?.body).toMatchObject({ kind: 'worktree-link', author: 'human', target: { c: 'worktrees', id: 'id-a' }, set: { item: 'I-3' } });
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  });

  it('a refusal stays in the dialog with the board sentence', async () => {
    await mountTab(board(), refused);
    await chooseAction('a', /^Link to an item/);
    const box = await dialog();
    fireEvent.click(within(box).getByRole('combobox', { name: 'Item' }));
    fireEvent.click(await screen.findByRole('option', { name: /^I-3/ }));
    fireEvent.click(within(box).getByRole('button', { name: 'Link' }));
    expect(await within(box).findByText('Refused: no item "I-3" on this board.')).toBeTruthy();
    expect(screen.queryByRole('dialog')).not.toBeNull();
  });
});

describe('I-85 Keep a worktree from the row menu', { tags: ['standard'] }, () => {
  it('needs a note, then posts worktree-keep with the note and the seven days', async () => {
    const { calls } = await mountTab(board());
    await chooseAction('a', /^Keep$/);
    const box = await dialog();
    const keep = within(box).getByRole<HTMLButtonElement>('button', { name: 'Keep' });
    expect(keep.disabled).toBe(true);
    fireEvent.change(within(box).getByRole('textbox', { name: 'Why keep it' }), { target: { value: '  review pending  ' } });
    expect(keep.disabled).toBe(false);
    fireEvent.click(keep);
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]?.body).toMatchObject({ kind: 'worktree-keep', author: 'human', target: { c: 'worktrees', id: 'id-a' }, set: { note: 'review pending', days: 7 } });
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  });

  it('Cancel posts nothing', async () => {
    const { calls } = await mountTab(board());
    await chooseAction('a', /^Keep$/);
    fireEvent.click(within(await dialog()).getByRole('button', { name: 'Cancel' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(calls).toEqual([]);
  });
});

describe('I-85 Unlink a worktree from the row menu', { tags: ['standard'] }, () => {
  it('posts worktree-link with a null item after the confirm, then closes', async () => {
    const { calls } = await mountTab(board());
    await chooseAction('b', /^Unlink from its item/);
    fireEvent.click(within(await dialog()).getByRole('button', { name: 'Unlink' }));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]?.body).toMatchObject({ kind: 'worktree-link', author: 'human', target: { c: 'worktrees', id: 'id-b' }, set: { item: null } });
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  });

  it('Cancel posts nothing, and only a link made by hand offers Unlink', async () => {
    const { calls } = await mountTab(board());
    await chooseAction('b', /^Unlink from its item/);
    fireEvent.click(within(await dialog()).getByRole('button', { name: 'Cancel' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(calls).toEqual([]);
    await openMenu('a');
    expect(hasAction(/^Unlink from its item/)).toBe(false);
    expect(hasAction(/^Link to an item/)).toBe(true);
  });

  it('a link Argus works out from the branch is not offered for Unlink', async () => {
    await mountTab(board());
    await openMenu('c');
    expect(hasAction(/^Unlink from its item/)).toBe(false);
  });
});
