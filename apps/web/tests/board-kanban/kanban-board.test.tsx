/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { openFilters, pickView } from '../helpers/filter-clauses';
import { DAY, MIN, ago, boardOf, cleanKanban, columnOf, item, mountKanban, thread } from './kanban-harness';

cleanKanban();

const finished = [item('I-8', { status: 'done', updatedAt: ago(2 * DAY) }), item('I-9', { status: 'dropped', updatedAt: ago(3 * DAY) })];

describe('S1 the Kanban columns', { tags: ['standard'] }, () => {
  it('hides done and dropped items by default, and the toggle counts them and opens them', async () => {
    await mountKanban({ items: [item('I-1', { status: 'doing' }), ...finished] });
    expect(await screen.findByRole('group', { name: 'Doing, 1 card' })).toBeTruthy();
    expect(screen.queryByRole('group', { name: /^Done,/ })).toBeNull();
    expect(screen.queryByText('Item number 8')).toBeNull();
    expect(screen.getByText(/^1 open \/ 3/)).toBeTruthy();
    const toggle = screen.getByRole('button', { name: 'Show done (2)' });
    expect(toggle.getAttribute('aria-pressed')).toBe('false');
    fireEvent.click(toggle);
    expect(within(columnOf('Done, 2 cards')).getByText('Item number 8')).toBeTruthy();
    expect(within(columnOf('Done, 2 cards')).getByText('Item number 9')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Hide done (2)' }).getAttribute('aria-pressed')).toBe('true');
    expect(localStorage.getItem('coord.done2')).toBe('1');
    fireEvent.click(screen.getByRole('button', { name: 'Hide done (2)' }));
    expect(screen.queryByText('Item number 8')).toBeNull();
    expect(localStorage.getItem('coord.done2')).toBe('0');
  });

  it('follows the stream: a status change moves the card, a new item appears', async () => {
    const { change } = await mountKanban({ items: [item('I-1')] });
    expect(within(await screen.findByRole('group', { name: 'To do, 1 card' })).getByText('Item number 1')).toBeTruthy();
    change('items', 'I-1', item('I-1', { status: 'review', rev: 4 }));
    expect(within(await screen.findByRole('group', { name: 'Review, 1 card' })).getByText('Item number 1')).toBeTruthy();
    change('items', 'I-2', item('I-2', { status: 'review' }));
    expect(await screen.findByRole('group', { name: 'Review, 2 cards' })).toBeTruthy();
  });

  it('groups by thread with a column for a listed thread that has no item', async () => {
    await mountKanban({ items: [item('I-1')], threads: [thread('worker'), thread('idle')] });
    await screen.findByText('Item number 1');
    await pickView(/^Group/, 'Thread');
    expect(columnOf('idle, 0 cards')).toBeTruthy();
    expect(within(columnOf('worker, 1 card')).getByText('Item number 1')).toBeTruthy();
    expect(localStorage.getItem('coord.g.items')).toBe('thread');
  });
});

describe('S1 what a card says about the owner', { tags: ['standard'] }, () => {
  it('says needs you for a question, a to-do and a manual test, and shows a chip for the open question and the decision to pick up', async () => {
    await mountKanban({
      items: [item('I-1'), item('I-2'), item('I-3'), item('I-4')],
      entries: [
        { id: 'Q-1', kind: 'question', item: 'I-1', author: 'worker', status: 'open', createdAt: ago(MIN) },
        { id: 'D-1', kind: 'decision', item: 'I-4', author: 'worker', status: 'pending', createdAt: ago(MIN) },
      ],
      actions: [{ id: 'A-1', item: 'I-2', status: 'pending' }],
      tests: [{ id: 'T-1', item: 'I-3', mode: 'manual', status: 'pending', title: 't', ac: [] }],
    });
    expect(within(await boardOf()).getAllByText('needs you')).toHaveLength(3);
    expect(screen.getByTitle('Q-1 is an open question, waiting for an answer')).toBeTruthy();
    expect(screen.getByTitle('D-1 is a decision to pick up')).toBeTruthy();
  });
});

describe('S1 the toolbar', { tags: ['standard'] }, () => {
  it('searches the id and the title', async () => {
    await mountKanban({ items: [item('I-1', { title: 'Login page' }), item('I-2', { title: 'Logout' }), item('I-3', { title: 'Other' })] });
    await openFilters();
    fireEvent.change(await screen.findByLabelText('Search id or title'), { target: { value: 'log' } });
    expect(screen.getByText('Login page')).toBeTruthy();
    expect(screen.getByText('Logout')).toBeTruthy();
    expect(screen.queryByText('Other')).toBeNull();
    expect(screen.getByText('3 open / 3 · filtered')).toBeTruthy();
  });

  it('clears the search to bring every item back', async () => {
    await mountKanban({ items: [item('I-1', { title: 'Login page' }), item('I-2', { title: 'Other' })] });
    await screen.findByText('Login page');
    await openFilters();
    fireEvent.change(screen.getByLabelText('Search id or title'), { target: { value: 'login' } });
    expect(screen.queryByText('Other')).toBeNull();
    fireEvent.change(screen.getByLabelText('Search id or title'), { target: { value: '' } });
    expect(screen.getByText('Other')).toBeTruthy();
  });
});

describe('K2 the filters stored for the Node page', { tags: ['standard'] }, () => {
  it('remembers the filters in the shape of the Node page, with the order inside them', async () => {
    await mountKanban({ items: [item('I-1')] });
    await openFilters();
    fireEvent.change(await screen.findByLabelText('Search id or title'), { target: { value: 'abc' } });
    fireEvent.click(screen.getByRole('checkbox', { name: 'Needs you' }));
    expect(JSON.parse(localStorage.getItem('coord.f') ?? '{}')).toMatchObject({ q: 'abc', needs: true, sort: 'rank' });
  });
});
