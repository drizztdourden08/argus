/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { MIN, ago, cleanKanban, item, mountKanban, openItem } from './kanban-harness';

cleanKanban();

describe('adding an item', () => {
  it('opens a dialog, keeps Add off until there is a title, and posts a create op for the first listed thread', async () => {
    const { posted } = await mountKanban({ items: [] });
    expect(screen.queryByRole('dialog', { name: 'New work item' })).toBeNull();
    fireEvent.click(await screen.findByRole('button', { name: 'New item' }));
    const dialog = screen.getByRole('dialog', { name: 'New work item' });
    expect(within(dialog).getByRole<HTMLButtonElement>('button', { name: 'Add' }).disabled).toBe(true);
    fireEvent.change(within(dialog).getByRole('textbox', { name: 'Title' }), { target: { value: '  Brand new  ' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Add' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'create', author: 'human', target: { c: 'items' }, set: { title: 'Brand new', thread: 'worker', status: 'todo' } });
    expect(await screen.findByText('Adding "Brand new".')).toBeTruthy();
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'New work item' })).toBeNull());
  });

  it('adds with Enter in the title field', async () => {
    const { posted } = await mountKanban({ items: [item('I-1')] });
    fireEvent.click(await screen.findByRole('button', { name: 'New item' }));
    const box = within(screen.getByRole('dialog', { name: 'New work item' })).getByRole('textbox', { name: 'Title' });
    fireEvent.change(box, { target: { value: 'By keyboard' } });
    fireEvent.keyDown(box, { key: 'Enter' });
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ set: { title: 'By keyboard' } });
  });

  it('does not add a title over 80 characters', async () => {
    const { posted } = await mountKanban({ items: [] });
    fireEvent.click(await screen.findByRole('button', { name: 'New item' }));
    const dialog = screen.getByRole('dialog', { name: 'New work item' });
    fireEvent.change(within(dialog).getByRole('textbox', { name: 'Title' }), { target: { value: 'x'.repeat(81) } });
    expect(within(dialog).getByRole<HTMLButtonElement>('button', { name: 'Add' }).disabled).toBe(true);
    expect(posted).toHaveLength(0);
  });
});

describe('picking several cards', () => {
  const two = [item('I-1'), item('I-2')];

  const pick = async () => {
    const mounted = await mountKanban({ items: two });
    fireEvent.click(await screen.findByRole('checkbox', { name: 'Select I-1' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'Select I-2' }));
    return { ...mounted, bar: screen.getByRole('toolbar', { name: 'Selected items' }) };
  };

  it('shows how many are picked, and nothing when none is', async () => {
    const { bar } = await pick();
    expect(within(bar).getByText('2 selected')).toBeTruthy();
    fireEvent.click(within(bar).getByRole('button', { name: 'Clear selection' }));
    expect(screen.queryByRole('toolbar', { name: 'Selected items' })).toBeNull();
  });

  it('archives them in one bulk entry, and the Undo sends the inverse with the id it undoes', async () => {
    const { posted, bar } = await pick();
    fireEvent.click(within(bar).getByRole('button', { name: 'Archive' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({
      kind: 'bulk',
      ops: [{ kind: 'archive', target: { c: 'items', id: 'I-1' } }, { kind: 'archive', target: { c: 'items', id: 'I-2' } }],
    });
    expect(await screen.findByText('2 changes sent.')).toBeTruthy();
    expect(screen.queryByRole('toolbar', { name: 'Selected items' })).toBeNull();
    fireEvent.click(await screen.findByRole('button', { name: /Undo/ }));
    await waitFor(() => expect(posted).toHaveLength(2));
    expect(posted[1]?.body).toMatchObject({
      kind: 'bulk',
      ops: [{ kind: 'unarchive', target: { c: 'items', id: 'I-1' } }, { kind: 'unarchive', target: { c: 'items', id: 'I-2' } }],
      undoes: posted[0]?.body.id,
    });
    expect(await screen.findByText('Undone.')).toBeTruthy();
  });

  it('deletes them with restore as the Undo', async () => {
    const { posted, bar } = await pick();
    fireEvent.click(within(bar).getByRole('button', { name: 'Delete' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'bulk', ops: [{ kind: 'delete' }, { kind: 'delete' }] });
    fireEvent.click(await screen.findByRole('button', { name: /Undo/ }));
    await waitFor(() => expect(posted).toHaveLength(2));
    expect(posted[1]?.body).toMatchObject({ kind: 'bulk', ops: [{ kind: 'restore' }, { kind: 'restore' }] });
  });

  it('forgets a pick when the card leaves the board', async () => {
    const { change } = await pick();
    change('items', 'I-1', item('I-1', { deletedAt: ago(MIN) }));
    await waitFor(() => expect(screen.getByText('1 selected')).toBeTruthy());
  });
});

describe('the menu of a card', () => {
  const openMenu = async (id: string) => {
    fireEvent.click(await screen.findByRole('button', { name: `More actions for ${id}` }));
  };

  it('opens the drawer from Open details', async () => {
    await mountKanban({ items: [item('I-1')] });
    await openMenu('I-1');
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Open details' }));
    expect(await screen.findByRole('dialog', { name: 'Work item' })).toBeTruthy();
  });

  it('archives and deletes with the same ops and Undo as the drawer', async () => {
    const { posted } = await mountKanban({ items: [item('I-1')] });
    await openMenu('I-1');
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Archive' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'archive', target: { c: 'items', id: 'I-1' } });
    await openMenu('I-1');
    fireEvent.click(await screen.findByRole('menuitem', { name: 'Delete' }));
    await waitFor(() => expect(posted).toHaveLength(2));
    expect(posted[1]?.body).toMatchObject({ kind: 'delete', target: { c: 'items', id: 'I-1' } });
  });

  it('closes the drawer with its Close button', async () => {
    await mountKanban({ items: [item('I-1')] });
    const dialog = await openItem('I-1');
    fireEvent.click(within(dialog).getByRole('button', { name: 'Close' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Work item' })).toBeNull());
  });
});
