/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { MIN, ago, cleanKanban, item, mountKanban, openItem, thread } from './kanban-harness';

cleanKanban();

describe('adding an item', () => {
  it('opens the inline form of a column, keeps Add off until there is a title, and posts a create op for the first listed thread', async () => {
    const { posted } = await mountKanban({ items: [] });
    expect(screen.queryByRole('textbox', { name: 'Title' })).toBeNull();
    fireEvent.click(await screen.findByRole('button', { name: 'Add a work item to To do' }));
    expect(screen.queryByRole('dialog', { name: 'New work item' })).toBeNull();
    expect(screen.getByRole<HTMLButtonElement>('button', { name: 'Add' }).disabled).toBe(true);
    fireEvent.change(screen.getByRole('textbox', { name: 'Title' }), { target: { value: '  Brand new  ' } });
    fireEvent.click(screen.getByRole('button', { name: 'Add' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'create', author: 'human', target: { c: 'items' }, set: { title: 'Brand new', thread: 'worker', status: 'todo' } });
    expect(await screen.findByText('Adding "Brand new".')).toBeTruthy();
    await waitFor(() => expect(screen.queryByRole('textbox', { name: 'Title' })).toBeNull());
  });

  it('adds with Enter in the title field', async () => {
    const { posted } = await mountKanban({ items: [item('I-1')] });
    fireEvent.click(await screen.findByRole('button', { name: 'Add a work item to To do' }));
    const box = screen.getByRole('textbox', { name: 'Title' });
    fireEvent.change(box, { target: { value: 'By keyboard' } });
    fireEvent.keyDown(box, { key: 'Enter' });
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ set: { title: 'By keyboard' } });
  });

  it('lets the thread be picked, and hides the thread choice in a thread column', async () => {
    const { posted, unmount } = await mountKanban({ items: [item('I-1')], threads: [thread('worker'), thread('idle')] });
    fireEvent.click(await screen.findByRole('button', { name: 'Add a work item to To do' }));
    expect(screen.getByRole('combobox', { name: 'Thread' })).toBeTruthy();
    fireEvent.change(screen.getByRole('textbox', { name: 'Title' }), { target: { value: 'For idle' } });
    fireEvent.click(screen.getByRole('combobox', { name: 'Thread' }));
    fireEvent.click(await screen.findByRole('option', { name: 'idle' }));
    fireEvent.click(screen.getByRole('button', { name: 'Add' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ set: { title: 'For idle', thread: 'idle' } });
    unmount();
    localStorage.setItem('coord.g.items', 'thread');
    await mountKanban({ items: [item('I-1')], threads: [thread('worker'), thread('idle')] });
    fireEvent.click(await screen.findByRole('button', { name: 'Add a work item to idle' }));
    expect(screen.getByRole('textbox', { name: 'Title' })).toBeTruthy();
    expect(screen.queryByRole('combobox', { name: 'Thread' })).toBeNull();
  });

  it('closes the form with Cancel and with a second click on + Add', async () => {
    await mountKanban({ items: [item('I-1')] });
    const open = await screen.findByRole('button', { name: 'Add a work item to To do' });
    fireEvent.click(open);
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByRole('textbox', { name: 'Title' })).toBeNull();
    fireEvent.click(open);
    expect(screen.getByRole('textbox', { name: 'Title' })).toBeTruthy();
    fireEvent.click(open);
    expect(screen.queryByRole('textbox', { name: 'Title' })).toBeNull();
  });

  it('does not add a title over 80 characters', async () => {
    const { posted } = await mountKanban({ items: [] });
    fireEvent.click(await screen.findByRole('button', { name: 'Add a work item to To do' }));
    fireEvent.change(screen.getByRole('textbox', { name: 'Title' }), { target: { value: 'x'.repeat(81) } });
    fireEvent.click(screen.getByRole('button', { name: 'Add' }));
    expect(await screen.findByText('At most 80 characters.')).toBeTruthy();
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
