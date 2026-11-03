/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { inTab, opened, rich } from './kanban-drawer-dom';
import { MIN, ago, cleanKanban, item, mountKanban, openItem } from './kanban-harness';

cleanKanban();

describe('S1 the detail drawer', { tags: ['standard'] }, () => {
  it('lists the criteria met, and the tests of this item only', async () => {
    const { dialog } = await opened();
    expect(within(dialog).getByText('Acceptance criteria · 1/2')).toBeTruthy();
    expect(within(dialog).getByRole<HTMLInputElement>('checkbox', { name: 'Criterion 1 met' }).checked).toBe(false);
    expect(within(dialog).getByRole<HTMLInputElement>('checkbox', { name: 'Criterion 2 met' }).checked).toBe(true);
    expect(within(dialog).getByText('proved by T-1')).toBeTruthy();
    inTab(dialog, 'Tests');
    expect(within(dialog).getByText('Tests · 1/2 passing')).toBeTruthy();
    expect(within(dialog).getByText('Wrong password')).toBeTruthy();
    expect(within(dialog).queryByText('Elsewhere')).toBeNull();
  });

  it('says needs you in the drawer while a question waits on the owner, and stops once it is answered', async () => {
    const question = { id: 'Q-1', kind: 'question', item: 'I-1', author: 'worker', status: 'open', createdAt: ago(10 * MIN) };
    const { change } = await mountKanban({ items: [item('I-1')], entries: [question] });
    const dialog = await openItem('I-1');
    expect(within(dialog).getByText('needs you')).toBeTruthy();
    change('entries', 'H-1', { id: 'H-1', kind: 'followup', author: 'human', replyTo: 'Q-1', status: 'new', createdAt: ago(MIN) });
    await waitFor(() => expect(within(dialog).queryByText('needs you')).toBeNull());
  });

  it('follows the stream while it is open', async () => {
    const { change, dialog } = await opened();
    change('items', 'I-1', rich({ title: 'Login v2', rev: 4 }));
    await waitFor(() => expect(within(dialog).getByText('Login v2')).toBeTruthy());
  });
});

describe('K4 the description of an item', { tags: ['standard'] }, () => {
  it('shows the description as formatted text, never as markup', async () => {
    const { dialog } = await opened({ description: 'Make **login** work. <b>raw</b>\n\n- one\n- two' });
    expect(within(dialog).getByText('login').tagName).toBe('STRONG');
    expect(within(dialog).getAllByRole('listitem').map((entry) => entry.textContent)).toEqual(['one', 'two']);
    expect(dialog.querySelector('b')).toBeNull();
  });
});

describe('K5 I-86 the tests a drawer counts', { tags: ['standard'] }, () => {
  it('counts no archived test and no test of a closed bundle on I-86, as the Node page does', async () => {
    const moved = { id: 'T-5', item: 'I-1', title: 'Moved away', mode: 'manual', status: 'moved', ac: [], archivedAt: ago(MIN) };
    const closed = { id: 'T-6', item: 'I-1', title: 'In a closed bundle', mode: 'manual', status: 'pass', ac: [], build: 'old-1' };
    await mountKanban({
      items: [item('I-86')],
      tests: [{ ...moved, item: 'I-86' }, { ...closed, item: 'I-86' }],
      builds: [{ tag: 'old-1', closedAt: ago(MIN) }],
    });
    const dialog = inTab(await openItem('I-86'), 'Tests');
    expect(within(dialog).getByText('Tests · 0/0 passing')).toBeTruthy();
    expect(within(dialog).queryByText('Moved away')).toBeNull();
  });
});

describe('K3 archive and delete from the drawer', { tags: ['standard'] }, () => {
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
