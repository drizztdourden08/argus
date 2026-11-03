/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { HOUR, MIN, ago, cleanKanban, item, moreItem, mountKanban } from './kanban-harness';
import type { Board } from './kanban-harness';

cleanKanban();

const trashed = (id: string, extra = {}) => item(id, { title: `Gone ${id}`, deletedAt: ago(HOUR), deletedBy: 'lead', ...extra });

const board: Board = {
  items: [item('I-1'), trashed('I-5'), trashed('I-6')],
  tests: [{ id: 'T-1', item: 'I-1', title: 'Old test', mode: 'auto', status: 'pass', ac: [], thread: 'worker', deletedAt: ago(2 * HOUR) }],
  entries: [{ id: 'Q-9', kind: 'question', item: 'I-1', thread: 'worker', author: 'worker', status: 'open', text: 'Dead question', createdAt: ago(HOUR), deletedAt: ago(30 * MIN) }],
};

const openTrash = async (extra: Board = {}, count = 4) => {
  const mounted = await mountKanban({ ...board, ...extra });
  await screen.findByText('Item number 1');
  await moreItem(`Trash (${count})`);
  return mounted;
};

describe('K3 the Trash view', { tags: ['standard'] }, () => {
  it('counts what was deleted and lists it with its kind, text, thread and who deleted it', async () => {
    await openTrash();
    const table = within(screen.getByRole('table', { name: 'Trash' }));
    expect(table.getByText('Gone I-5')).toBeTruthy();
    expect(table.getByText('Old test')).toBeTruthy();
    expect(table.getByText('Dead question')).toBeTruthy();
    expect(table.getAllByText('item')).toHaveLength(2);
    expect(table.getAllByText(/^\d+[mh] · lead$/).length).toBeGreaterThan(0);
    expect(screen.queryByText('Item number 1')).toBeNull();
  });

  it('opens from the More menu, which counts the trash, and Back to the board closes it', async () => {
    await mountKanban(board);
    await screen.findByText('Item number 1');
    expect(screen.queryByRole('button', { name: /^Trash \(/ })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'More' }));
    expect(await screen.findByRole('menuitem', { name: 'Trash (4)' })).toBeTruthy();
    fireEvent.click(screen.getByRole('menuitem', { name: 'Trash (4)' }));
    expect(screen.getByRole('table', { name: 'Trash' })).toBeTruthy();
    expect(screen.queryByRole('search', { name: 'Filters' })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Back to the board' }));
    expect(screen.getByText('Item number 1')).toBeTruthy();
  });

  it('restores one row with a restore op that names its collection, and Undo deletes it again', async () => {
    const { posted } = await openTrash();
    fireEvent.click(screen.getByRole('button', { name: 'Restore T-1' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'bulk', ops: [{ kind: 'restore', target: { c: 'tests', id: 'T-1' } }] });
    fireEvent.click(await screen.findByRole('button', { name: /Undo/ }));
    await waitFor(() => expect(posted).toHaveLength(2));
    expect(posted[1]?.body.ops).toEqual([{ kind: 'delete', target: { c: 'tests', id: 'T-1' } }]);
  });

  it('brings back an archived agent with unarchive', async () => {
    const { posted } = await openTrash({ agents: [{ id: 'A-1', name: 'coder', task: 'build it', archivedAt: ago(HOUR), thread: 'worker' }] }, 5);
    fireEvent.click(await screen.findByRole('button', { name: 'Restore A-1' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ ops: [{ kind: 'unarchive', target: { c: 'agents', id: 'A-1' } }] });
  });
});

describe('K4 the Purge dialog', { tags: ['standard'] }, () => {
  it('counts what a purge of the trash removes and asks for the project key', async () => {
    const { posted } = await openTrash();
    fireEvent.click(screen.getByRole('button', { name: 'Empty trash...' }));
    const dialog = within(await screen.findByRole('dialog', { name: 'Purge' }));
    expect(dialog.getByRole<HTMLInputElement>('radio', { name: 'Empty the trash (every deleted record)' }).checked).toBe(true);
    const confirm = dialog.getByRole<HTMLButtonElement>('button', { name: 'Purge' });
    expect(confirm.disabled).toBe(true);
    fireEvent.change(dialog.getByRole('textbox', { name: /Type the project key alpha/ }), { target: { value: 'alpha' } });
    expect(confirm.disabled).toBe(false);
    fireEvent.click(confirm);
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({
      kind: 'purge',
      criteria: { trash: true },
      expect: { threads: 0, items: 2, ac: 0, tests: 1, entries: 1, actions: 0, logLines: 0 },
      confirm: 'alpha',
    });
  });
});

describe('K3 what the board answered to a purge', { tags: ['standard'] }, () => {
  it('shows an applied or refused purge with a Dismiss that acknowledges it', async () => {
    const entry = { id: 'O-1', kind: 'purge', author: 'human', status: 'refused', outcome: 'the counts changed', createdAt: ago(MIN) };
    const { posted } = await mountKanban({ items: [item('I-1')], entries: [entry] });
    expect(await screen.findByText('Purge refused the counts changed')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'ack', replyTo: 'O-1' });
  });
});
