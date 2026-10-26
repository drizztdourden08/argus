/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { opened } from './kanban-drawer-dom';
import { cleanKanban } from './kanban-harness';

cleanKanban();

describe('the criteria of an item: ticking and adding', () => {
  it('ticks a criterion with a done op and unticks with an undo op', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('checkbox', { name: 'Criterion 1 met' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'ac', target: { c: 'items', id: 'I-1' }, do: 'done', n: 1, baseRev: 3 });
    fireEvent.click(within(dialog).getByRole('checkbox', { name: 'Criterion 2 met' }));
    await waitFor(() => expect(posted).toHaveLength(2));
    expect(posted[1]?.body).toMatchObject({ kind: 'ac', do: 'undo', n: 2, baseRev: 3 });
  });

  it('adds a criterion with Enter or the button, and says so', async () => {
    const { posted, dialog } = await opened();
    const box = within(dialog).getByRole('textbox', { name: 'Add a criterion' });
    fireEvent.change(box, { target: { value: 'Locks after three tries' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Add criterion' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'ac', do: 'add', text: 'Locks after three tries', baseRev: 3 });
    expect(await screen.findByText('Criterion added to I-1.')).toBeTruthy();
    await waitFor(() => expect(within(dialog).getByRole<HTMLInputElement>('textbox', { name: 'Add a criterion' }).value).toBe(''));
  });

  it('refuses an empty criterion and one over 120 characters, posting nothing', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Add criterion' }));
    expect(within(dialog).getByRole('alert').textContent).toBe('A criterion cannot be empty.');
    fireEvent.change(within(dialog).getByRole('textbox', { name: 'Add a criterion' }), { target: { value: 'x'.repeat(121) } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Add criterion' }));
    expect(within(dialog).getByRole('alert').textContent).toBe('At most 120 characters.');
    expect(posted).toHaveLength(0);
  });
});

describe('the criteria of an item: editing, removing and moving', () => {
  it('edits a criterion with the text it had as prev', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit criterion 1: Shows an error' }));
    const box = within(dialog).getByRole('textbox', { name: 'Edit criterion 1' });
    fireEvent.change(box, { target: { value: 'Shows a clear error' } });
    fireEvent.keyDown(box, { key: 'Enter' });
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'ac', target: { c: 'items', id: 'I-1' }, do: 'set', n: 1, text: 'Shows a clear error', prev: { text: 'Shows an error' }, baseRev: 3 });
    expect(await screen.findByText('Saved I-1 criterion 1.')).toBeTruthy();
  });

  it('keeps the old text for an edit that changes nothing, and does not post', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit criterion 1: Shows an error' }));
    fireEvent.keyDown(within(dialog).getByRole('textbox', { name: 'Edit criterion 1' }), { key: 'Enter' });
    expect(within(dialog).queryByRole('textbox', { name: 'Edit criterion 1' })).toBeNull();
    expect(posted).toHaveLength(0);
  });

  it('cancels an edit with Escape', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Edit criterion 1: Shows an error' }));
    const box = within(dialog).getByRole('textbox', { name: 'Edit criterion 1' });
    fireEvent.change(box, { target: { value: 'changed' } });
    fireEvent.keyDown(box, { key: 'Escape' });
    expect(within(dialog).queryByRole('textbox', { name: 'Edit criterion 1' })).toBeNull();
    expect(within(dialog).getByText('Shows an error')).toBeTruthy();
    expect(posted).toHaveLength(0);
  });

  it('removes a criterion with the text it had as prev', async () => {
    const { posted, dialog } = await opened();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Remove criterion 2' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'ac', do: 'rm', n: 2, prev: { text: 'Keeps the session' }, baseRev: 3 });
    expect(await screen.findByText('Criterion 2 of I-1 removed.')).toBeTruthy();
  });

  it('moves a criterion with the arrow keys on its grip, and not past either end', async () => {
    const { posted, dialog } = await opened();
    fireEvent.keyDown(within(dialog).getByRole('button', { name: 'Move criterion 1' }), { key: 'ArrowUp' });
    fireEvent.keyDown(within(dialog).getByRole('button', { name: 'Move criterion 2' }), { key: 'ArrowDown' });
    expect(posted).toHaveLength(0);
    fireEvent.keyDown(within(dialog).getByRole('button', { name: 'Move criterion 2' }), { key: 'ArrowUp' });
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'ac', do: 'move', n: 2, to: 1, baseRev: 3 });
    fireEvent.keyDown(within(dialog).getByRole('button', { name: 'Move criterion 1' }), { key: 'ArrowDown' });
    await waitFor(() => expect(posted).toHaveLength(2));
    expect(posted[1]?.body).toMatchObject({ do: 'move', n: 1, to: 2 });
    expect(await within(dialog).findByText('Criterion 1 moved to 2 of 2')).toBeTruthy();
  });
});
