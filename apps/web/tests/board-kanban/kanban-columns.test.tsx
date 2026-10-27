/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { HOUR, ago, boardOf, cleanKanban, columnOf, item, mountKanban, thread } from './kanban-harness';

cleanKanban();

describe('the column header: + Add and the quiet chip', () => {
  it('counts the quiet items of a column in a chip with its reason', async () => {
    await mountKanban({
      items: [
        item('I-1', { status: 'doing', updatedAt: ago(9 * HOUR) }),
        item('I-2', { status: 'doing', updatedAt: ago(HOUR) }),
        item('I-3', { status: 'review', updatedAt: ago(8 * HOUR) }),
        item('I-4', { status: 'review', updatedAt: ago(2 * HOUR) }),
      ],
    });
    const doing = within(await screen.findByRole('group', { name: 'Doing, 2 cards' }));
    expect(doing.getByText('1 quiet')).toBeTruthy();
    expect(doing.getByTitle(/^1 item has had no change for too long/)).toBeTruthy();
    expect(within(columnOf('Review, 2 cards')).getByText('1 quiet')).toBeTruthy();
    expect(screen.queryByText('2 quiet')).toBeNull();
  });

  it('adds an item to a status column with that status', async () => {
    const { posted } = await mountKanban({ items: [item('I-1')] });
    await screen.findByText('Item I-1');
    fireEvent.click(screen.getByRole('button', { name: 'Add a work item to Doing' }));
    fireEvent.change(await screen.findByRole('textbox', { name: 'Title' }), { target: { value: 'New thing' } });
    fireEvent.click(screen.getByRole('button', { name: 'Add' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'create', target: { c: 'items' }, set: { title: 'New thing', thread: 'worker', status: 'doing' } });
  });

  it('adds an item to a thread column for that thread, and to a subject column with that subject', async () => {
    localStorage.setItem('coord.g.items', 'thread');
    const first = await mountKanban({ items: [item('I-1')], threads: [thread('worker'), thread('idle')] });
    await screen.findByText('Item I-1');
    fireEvent.click(screen.getByRole('button', { name: 'Add a work item to idle' }));
    fireEvent.change(await screen.findByRole('textbox', { name: 'Title' }), { target: { value: 'For idle' } });
    fireEvent.click(screen.getByRole('button', { name: 'Add' }));
    await waitFor(() => expect(first.posted).toHaveLength(1));
    expect(first.posted[0]?.body).toMatchObject({ set: { title: 'For idle', thread: 'idle', status: 'todo' } });
    first.unmount();
    localStorage.setItem('coord.g.items', 'subject');
    const second = await mountKanban({ items: [item('I-1', { subject: 'ui' })] });
    await screen.findByText('Item I-1');
    fireEvent.click(screen.getByRole('button', { name: 'Add a work item to ui' }));
    fireEvent.change(await screen.findByRole('textbox', { name: 'Title' }), { target: { value: 'Style it' } });
    fireEvent.click(screen.getByRole('button', { name: 'Add' }));
    await waitFor(() => expect(second.posted).toHaveLength(1));
    expect(second.posted[0]?.body).toMatchObject({ set: { title: 'Style it', thread: 'worker', subject: 'ui' } });
  });
});

describe('selecting a range of cards', () => {
  it('selects every card between the last click and a shift-click, in the order of the board', async () => {
    await mountKanban({ items: [item('I-1', { rank: 1 }), item('I-2', { rank: 2 }), item('I-3', { rank: 3 }), item('I-4', { rank: 4 })] });
    await screen.findByText('Item I-1');
    fireEvent.click(screen.getByRole('checkbox', { name: 'Select I-1' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'Select I-3' }), { shiftKey: true });
    expect(screen.getByText('3 selected')).toBeTruthy();
    expect(screen.getByRole<HTMLInputElement>('checkbox', { name: 'Select I-2' }).checked).toBe(true);
    expect(screen.getByRole<HTMLInputElement>('checkbox', { name: 'Select I-4' }).checked).toBe(false);
  });

  it('a click without shift picks one card only', async () => {
    await mountKanban({ items: [item('I-1', { rank: 1 }), item('I-2', { rank: 2 }), item('I-3', { rank: 3 })] });
    await screen.findByText('Item I-1');
    fireEvent.click(screen.getByRole('checkbox', { name: 'Select I-1' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'Select I-3' }));
    expect(screen.getByText('2 selected')).toBeTruthy();
    expect(screen.getByRole<HTMLInputElement>('checkbox', { name: 'Select I-2' }).checked).toBe(false);
  });
});

describe('paused threads and the focus', () => {
  const board = {
    items: [item('I-1', { thread: 'worker' }), item('I-2', { thread: 'sleeper' })],
    threads: [thread('worker'), thread('sleeper', { status: 'paused' })],
  };

  it('hides the items of a paused thread, and shows them with the toggle', async () => {
    await mountKanban(board);
    expect(await screen.findByText('Item I-1')).toBeTruthy();
    expect(screen.queryByText('Item I-2')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Show paused (1)' }));
    expect(screen.getByText('Item I-2')).toBeTruthy();
    expect(localStorage.getItem('coord.paused')).toBe('1');
  });

  it('counts a pause the owner posted that the board has not applied yet', async () => {
    await mountKanban({
      items: board.items,
      threads: [thread('worker'), thread('sleeper')],
      entries: [{ id: 'O-1', kind: 'pause', author: 'human', replyTo: 'sleeper', status: 'new', createdAt: ago(1000) }],
    });
    await screen.findByText('Item I-1');
    expect(screen.queryByText('Item I-2')).toBeNull();
  });

  it('shows only the focused thread, and Show all clears the focus', async () => {
    await mountKanban({ items: [item('I-1', { thread: 'worker' }), item('I-2', { thread: 'other' })], threads: [thread('worker'), thread('other')] });
    await screen.findByText('Item I-1');
    fireEvent.click(screen.getByRole('combobox', { name: 'Focus' }));
    fireEvent.click(await screen.findByRole('option', { name: 'other' }));
    expect(screen.queryByText('Item I-1')).toBeNull();
    expect(within(await boardOf()).getByText('Item I-2')).toBeTruthy();
    expect(localStorage.getItem('coord.focus')).toBe('other');
    expect(screen.getByText('Only other is shown. Its open questions are in Activity.')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Show all' }));
    expect(screen.getByText('Item I-1')).toBeTruthy();
    expect(localStorage.getItem('coord.focus')).toBe('');
  });

  it('forgets a saved focus on a thread that is gone', async () => {
    localStorage.setItem('coord.focus', 'ghost');
    await mountKanban({ items: [item('I-1')] });
    expect(await screen.findByText('Item I-1')).toBeTruthy();
  });
});
