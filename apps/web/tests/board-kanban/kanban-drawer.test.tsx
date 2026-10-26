/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { opened, rich } from './kanban-drawer-dom';
import { cleanKanban } from './kanban-harness';

cleanKanban();

describe('the detail drawer', () => {
  it('opens from the card title and names the item', async () => {
    const { dialog } = await opened();
    expect(within(dialog).getByText('I-1')).toBeTruthy();
    expect(within(dialog).getByRole('heading', { name: /Login/ })).toBeTruthy();
  });

  it('shows the description as formatted text, never as markup', async () => {
    const { dialog } = await opened({ description: 'Make **login** work. <b>raw</b>\n\n- one\n- two' });
    expect(within(dialog).getByText('login').tagName).toBe('STRONG');
    expect(within(dialog).getAllByRole('listitem').map((entry) => entry.textContent)).toEqual(['one', 'two']);
    expect(dialog.querySelector('b')).toBeNull();
  });

  it('says there is no description yet', async () => {
    const { dialog } = await opened({ description: '' });
    expect(within(dialog).getByText('No description yet.')).toBeTruthy();
  });

  it('lists the criteria met, and the tests of this item only', async () => {
    const { dialog } = await opened();
    expect(within(dialog).getByText('Acceptance criteria, 1/2 met')).toBeTruthy();
    expect(within(dialog).getByRole<HTMLInputElement>('checkbox', { name: 'Criterion 1 met' }).checked).toBe(false);
    expect(within(dialog).getByRole<HTMLInputElement>('checkbox', { name: 'Criterion 2 met' }).checked).toBe(true);
    expect(within(dialog).getByText('proved by T-1')).toBeTruthy();
    expect(within(dialog).getByText('Tests, 1/2 passing')).toBeTruthy();
    expect(within(dialog).getByText('Wrong password')).toBeTruthy();
    expect(within(dialog).queryByText('Elsewhere')).toBeNull();
  });

  it('shows the revision line', async () => {
    const { dialog } = await opened({ rev: 5, updatedBy: 'lead' });
    expect(within(dialog).getByText(/^rev 5, updated by lead /)).toBeTruthy();
  });

  it('follows the stream while it is open', async () => {
    const { change, dialog } = await opened();
    change('items', 'I-1', rich({ title: 'Login v2', rev: 4 }));
    await waitFor(() => expect(within(dialog).getByText('Login v2')).toBeTruthy());
  });
});

describe('archive and delete from the drawer', () => {
  it('archives, and the Undo toast sends unarchive with the id of the entry it undoes', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Archive' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'archive', target: { c: 'items', id: 'I-1' } });
    expect(await screen.findByText('Archived I-1.')).toBeTruthy();
    fireEvent.click(await screen.findByRole('button', { name: /Undo/ }));
    await waitFor(() => expect(posted).toHaveLength(2));
    expect(posted[1]?.body).toMatchObject({ kind: 'unarchive', target: { c: 'items', id: 'I-1' }, undoes: posted[0]?.body.id });
  });

  it('offers Unarchive for an archived item', async () => {
    localStorage.setItem('coord.f', JSON.stringify({ archived: true }));
    const { posted, dialog } = await opened({ archivedAt: '2026-10-25T10:00:00.000Z' });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Unarchive' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'unarchive' });
  });

  it('deletes, and the Undo toast sends restore', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Delete' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'delete', target: { c: 'items', id: 'I-1' } });
    fireEvent.click(await screen.findByRole('button', { name: /Undo/ }));
    await waitFor(() => expect(posted).toHaveLength(2));
    expect(posted[1]?.body).toMatchObject({ kind: 'restore', undoes: posted[0]?.body.id });
  });
});
