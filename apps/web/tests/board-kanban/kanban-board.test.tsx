/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { DAY, MIN, ago, boardOf, cleanKanban, columnOf, item, mountKanban, thread } from './kanban-harness';

cleanKanban();

const finished = [item('I-8', { status: 'done', updatedAt: ago(2 * DAY) }), item('I-9', { status: 'dropped', updatedAt: ago(3 * DAY) })];

describe('S1 the Kanban columns', { tags: ['standard'] }, () => {
  it('hides done and dropped items by default, and the toggle counts them and opens them', async () => {
    await mountKanban({ items: [item('I-1', { status: 'doing' }), ...finished] });
    expect(await screen.findByRole('group', { name: 'Doing, 1 card' })).toBeTruthy();
    expect(screen.queryByRole('group', { name: /^Done,/ })).toBeNull();
    expect(screen.queryByText('Item I-8')).toBeNull();
    expect(screen.getByText(/^1 open \/ 3/)).toBeTruthy();
    const toggle = screen.getByRole('button', { name: 'Show done (2)' });
    expect(toggle.getAttribute('aria-pressed')).toBe('false');
    fireEvent.click(toggle);
    expect(within(columnOf('Done, 2 cards')).getByText('Item I-8')).toBeTruthy();
    expect(within(columnOf('Done, 2 cards')).getByText('Item I-9')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Hide done (2)' }).getAttribute('aria-pressed')).toBe('true');
    expect(localStorage.getItem('coord.done2')).toBe('1');
    fireEvent.click(screen.getByRole('button', { name: 'Hide done (2)' }));
    expect(screen.queryByText('Item I-8')).toBeNull();
    expect(localStorage.getItem('coord.done2')).toBe('0');
  });

  it('follows the stream: a status change moves the card, a new item appears', async () => {
    const { change } = await mountKanban({ items: [item('I-1')] });
    expect(within(await screen.findByRole('group', { name: 'To do, 1 card' })).getByText('Item I-1')).toBeTruthy();
    change('items', 'I-1', item('I-1', { status: 'review', rev: 4 }));
    expect(within(await screen.findByRole('group', { name: 'Review, 1 card' })).getByText('Item I-1')).toBeTruthy();
    change('items', 'I-2', item('I-2', { status: 'review' }));
    expect(await screen.findByRole('group', { name: 'Review, 2 cards' })).toBeTruthy();
  });

  it('groups by thread with a column for a listed thread that has no item', async () => {
    await mountKanban({ items: [item('I-1')], threads: [thread('worker'), thread('idle')] });
    await screen.findByText('Item I-1');
    fireEvent.click(screen.getByRole('radio', { name: 'Thread' }));
    expect(columnOf('idle, 0 cards')).toBeTruthy();
    expect(within(columnOf('worker, 1 card')).getByText('Item I-1')).toBeTruthy();
    expect(localStorage.getItem('coord.g.items')).toBe('thread');
  });
});

describe('S1 what a card says about the owner', { tags: ['standard'] }, () => {
  it('says needs you for a question, a to-do and a manual test, and counts the decisions to pick up', async () => {
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
    expect(screen.getByText('1 to decide')).toBeTruthy();
  });
});

describe('S1 the toolbar', { tags: ['standard'] }, () => {
  it('searches the id and the title', async () => {
    await mountKanban({ items: [item('I-1', { title: 'Login page' }), item('I-2', { title: 'Logout' }), item('I-3', { title: 'Other' })] });
    fireEvent.change(await screen.findByLabelText('Search id or title'), { target: { value: 'log' } });
    expect(screen.getByText('Login page')).toBeTruthy();
    expect(screen.getByText('Logout')).toBeTruthy();
    expect(screen.queryByText('Other')).toBeNull();
    expect(screen.getByText('3 open / 3 · filtered')).toBeTruthy();
  });

  it('clears the filters with one button, which only shows while one is on', async () => {
    await mountKanban({ items: [item('I-1', { title: 'Login page' }), item('I-2', { title: 'Other' })] });
    await screen.findByText('Login page');
    expect(screen.queryByRole('button', { name: 'Clear filters' })).toBeNull();
    fireEvent.change(screen.getByLabelText('Search id or title'), { target: { value: 'login' } });
    expect(screen.queryByText('Other')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Clear filters' }));
    expect(screen.getByText('Other')).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Clear filters' })).toBeNull();
  });
});

describe('K2 the filters stored for the Node page', { tags: ['standard'] }, () => {
  it('remembers the filters in the shape of the Node page, with the order inside them', async () => {
    await mountKanban({ items: [item('I-1')] });
    fireEvent.change(await screen.findByLabelText('Search id or title'), { target: { value: 'abc' } });
    fireEvent.click(screen.getByRole('checkbox', { name: 'needs you' }));
    expect(JSON.parse(localStorage.getItem('coord.f') ?? '{}')).toMatchObject({ q: 'abc', needs: true, sort: 'rank' });
  });
});
