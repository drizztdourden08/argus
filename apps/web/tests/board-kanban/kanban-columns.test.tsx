/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { boardOf, cardOf, cleanKanban, item, mountKanban, thread } from './kanban-harness';

cleanKanban();

describe('S1 the column header: + Add', { tags: ['standard'] }, () => {
  it('adds an item to a status column with that status', async () => {
    const { posted } = await mountKanban({ items: [item('I-1')] });
    await screen.findByText('Item I-1');
    fireEvent.click(screen.getByRole('button', { name: 'Add a work item to Doing' }));
    fireEvent.change(await screen.findByRole('textbox', { name: 'Title' }), { target: { value: 'New thing' } });
    fireEvent.click(screen.getByRole('button', { name: 'Add' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'create', target: { c: 'items' }, set: { title: 'New thing', thread: 'worker', status: 'doing' } });
  });
});

describe('S1 selecting a range of cards', { tags: ['standard'] }, () => {
  it('selects every card between the last click and a shift-click, in the order of the board', async () => {
    await mountKanban({ items: [item('I-1', { rank: 1 }), item('I-2', { rank: 2 }), item('I-3', { rank: 3 }), item('I-4', { rank: 4 })] });
    await screen.findByText('Item I-1');
    fireEvent.click(screen.getByRole('button', { name: 'Select' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'Select I-1' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'Select I-3' }), { shiftKey: true });
    expect(screen.getByText('3 selected')).toBeTruthy();
    expect(screen.getByRole<HTMLInputElement>('checkbox', { name: 'Select I-2' }).checked).toBe(true);
    expect(screen.getByRole<HTMLInputElement>('checkbox', { name: 'Select I-4' }).checked).toBe(false);
  });
});

describe('I-72 the selection mode of the cards', { tags: ['standard'] }, () => {
  it('shows a type mark and no checkbox until the Select toggle is on', async () => {
    await mountKanban({ items: [item('I-1')] });
    await screen.findByText('Item I-1');
    expect(screen.queryByRole('checkbox', { name: 'Select I-1' })).toBeNull();
    expect(within(cardOf('I-1')).getByRole('img', { name: 'Task' })).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Select' }));
    expect(screen.getByRole('checkbox', { name: 'Select I-1' })).toBeTruthy();
    expect(within(cardOf('I-1')).queryByRole('img', { name: 'Task' })).toBeNull();
  });

  it('turns the mode on with a shift-click on a card and picks it', async () => {
    await mountKanban({ items: [item('I-1'), item('I-2')] });
    await screen.findByText('Item I-1');
    fireEvent.click(document.querySelector('[data-item="I-1"]') as HTMLElement, { shiftKey: true });
    expect(screen.getByText('1 selected')).toBeTruthy();
    expect(screen.getByRole<HTMLInputElement>('checkbox', { name: 'Select I-1' }).checked).toBe(true);
  });
});

describe('S1 paused threads and the focus', { tags: ['standard'] }, () => {
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
});

describe('S3 a saved focus on a thread that is gone', { tags: ['standard'] }, () => {
  it('forgets the focus and shows the board', async () => {
    localStorage.setItem('coord.focus', 'ghost');
    await mountKanban({ items: [item('I-1')] });
    expect(await screen.findByText('Item I-1')).toBeTruthy();
  });
});
