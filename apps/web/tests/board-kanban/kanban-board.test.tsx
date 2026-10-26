/* @layer renderer-app @kind test */
import { fireEvent, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { DAY, HOUR, MIN, ago, boardOf, cleanKanban, columnOf, item, mountKanban, thread } from './kanban-harness';

cleanKanban();

const finished = [item('I-8', { status: 'done', updatedAt: ago(2 * DAY) }), item('I-9', { status: 'dropped', updatedAt: ago(3 * DAY) })];

describe('the Kanban columns', () => {
  it('puts the work items in the columns of their status, with the count in the column name', async () => {
    await mountKanban({
      items: [item('I-1'), item('I-2', { status: 'doing' }), item('I-3', { status: 'doing' }), item('I-4', { status: 'review' })],
    });
    expect(await screen.findByRole('group', { name: 'Doing, 2 cards' })).toBeTruthy();
    expect(within(columnOf('Doing, 2 cards')).getByText('Item I-2')).toBeTruthy();
    expect(within(columnOf('Doing, 2 cards')).getByText('Item I-3')).toBeTruthy();
    expect(within(columnOf('To do, 1 card')).getByText('Item I-1')).toBeTruthy();
    expect(within(columnOf('Review, 1 card')).getByText('Item I-4')).toBeTruthy();
  });

  it('shows the id, the thread, the criteria met and the age on a card', async () => {
    await mountKanban({
      items: [item('I-1', { thread: 'worker', ac: [{ text: 'a', done: true }, { text: 'b', done: false }], updatedAt: ago(5 * MIN) })],
    });
    const card = within(await screen.findByRole('group', { name: 'To do, 1 card' }));
    expect(card.getByText('I-1')).toBeTruthy();
    expect(card.getByText('worker')).toBeTruthy();
    expect(card.getByText('1/2')).toBeTruthy();
    expect(card.getByText('5m')).toBeTruthy();
  });

  it('says there is no work item when no column exists at all', async () => {
    localStorage.setItem('coord.g.items', 'subject');
    await mountKanban({ items: [] });
    expect(await screen.findByText('No work item yet. Threads enter their own as they take work on.')).toBeTruthy();
  });

  it('hides done and dropped items by default, and the toggle counts them and opens them', async () => {
    await mountKanban({ items: [item('I-1', { status: 'doing' }), ...finished] });
    expect(await screen.findByRole('group', { name: 'Doing, 1 card' })).toBeTruthy();
    expect(screen.queryByRole('group', { name: /^Done,/ })).toBeNull();
    expect(screen.queryByText('Item I-8')).toBeNull();
    expect(screen.getByText(/^1 open \/ 3/)).toBeTruthy();
    const toggle = screen.getByRole('switch', { name: 'Show done (2)' });
    expect(toggle.getAttribute('aria-checked')).toBe('false');
    fireEvent.click(toggle);
    expect(within(columnOf('Done, 2 cards')).getByText('Item I-8')).toBeTruthy();
    expect(within(columnOf('Done, 2 cards')).getByText('Item I-9')).toBeTruthy();
    expect(screen.getByRole('switch', { name: 'Show done (2)' }).getAttribute('aria-checked')).toBe('true');
    expect(localStorage.getItem('coord.done2')).toBe('1');
    fireEvent.click(screen.getByRole('switch', { name: 'Show done (2)' }));
    expect(screen.queryByText('Item I-8')).toBeNull();
    expect(localStorage.getItem('coord.done2')).toBe('0');
  });

  it('opens the done items on load when the choice was saved, and still counts only the outstanding ones', async () => {
    localStorage.setItem('coord.done2', '1');
    await mountKanban({ items: [item('I-1', { status: 'doing' }), ...finished] });
    expect(await screen.findByText('Item I-8')).toBeTruthy();
    expect(screen.getByText(/^1 open \/ 3/)).toBeTruthy();
  });

  it('follows the stream: a status change moves the card, a new item appears', async () => {
    const { change } = await mountKanban({ items: [item('I-1')] });
    expect(within(await screen.findByRole('group', { name: 'To do, 1 card' })).getByText('Item I-1')).toBeTruthy();
    change('items', 'I-1', item('I-1', { status: 'review', rev: 4 }));
    expect(within(await screen.findByRole('group', { name: 'Review, 1 card' })).getByText('Item I-1')).toBeTruthy();
    change('items', 'I-2', item('I-2', { status: 'review' }));
    expect(await screen.findByRole('group', { name: 'Review, 2 cards' })).toBeTruthy();
  });
});

describe('the quiet mark', () => {
  it('puts a quiet tag with the age and its reason on a card with no change past its limit, and counts them', async () => {
    await mountKanban({
      items: [
        item('I-1', { status: 'doing', updatedAt: ago(5 * HOUR) }),
        item('I-2', { status: 'doing', updatedAt: ago(3 * HOUR) }),
        item('I-3', { status: 'todo', updatedAt: ago(2 * DAY) }),
      ],
    });
    expect(await screen.findByText('quiet 5h')).toBeTruthy();
    expect(screen.getByTitle(/^No change for 5h \(a doing item is reminded after 4 h\)\. Its thread must start it/)).toBeTruthy();
    expect(screen.getByText('quiet 2d')).toBeTruthy();
    expect(within(columnOf('Doing, 2 cards')).getAllByText(/^quiet /)).toHaveLength(1);
    expect(screen.getByText(/^3 open \/ 3 · 2 quiet$/)).toBeTruthy();
  });

  it('does not mark an item that waits on the owner', async () => {
    await mountKanban({
      items: [item('I-1', { status: 'doing', updatedAt: ago(2 * DAY) })],
      entries: [{ id: 'Q-1', kind: 'question', item: 'I-1', thread: 'worker', author: 'worker', status: 'open', text: 'which?', createdAt: ago(2 * DAY) }],
    });
    expect(within(await boardOf()).getByText('needs you')).toBeTruthy();
    expect(screen.queryByText(/^quiet /)).toBeNull();
  });
});

describe('what a card says about the owner', () => {
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

  it('stops saying it once the owner answered the question', async () => {
    await mountKanban({
      items: [item('I-1')],
      entries: [
        { id: 'Q-1', kind: 'question', item: 'I-1', author: 'worker', status: 'open', createdAt: ago(HOUR) },
        { id: 'H-1', kind: 'followup', author: 'human', replyTo: 'Q-1', status: 'new', createdAt: ago(MIN) },
      ],
    });
    expect(await screen.findByText('Item I-1')).toBeTruthy();
    expect(within(await boardOf()).queryByText('needs you')).toBeNull();
  });

  it('shows the status on the card when the grouping is not by status', async () => {
    localStorage.setItem('coord.g.items', 'thread');
    await mountKanban({ items: [item('I-1', { status: 'review' })] });
    expect(within(await screen.findByRole('group', { name: 'worker, 1 card' })).getByText('review')).toBeTruthy();
  });
});

describe('the toolbar', () => {
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

  it('remembers the filters in the shape of the Node page, with the order inside them', async () => {
    await mountKanban({ items: [item('I-1')] });
    fireEvent.change(await screen.findByLabelText('Search id or title'), { target: { value: 'abc' } });
    fireEvent.click(screen.getByRole('switch', { name: 'needs you' }));
    expect(JSON.parse(localStorage.getItem('coord.f') ?? '{}')).toMatchObject({ q: 'abc', needs: true, sort: 'rank' });
  });

  it('shows the archived items, marked, when asked', async () => {
    await mountKanban({ items: [item('I-1'), item('I-2', { archivedAt: ago(HOUR) })] });
    await screen.findByText('Item I-1');
    expect(screen.queryByText('Item I-2')).toBeNull();
    fireEvent.click(screen.getByRole('switch', { name: 'archived' }));
    expect(screen.getByText('Item I-2')).toBeTruthy();
    expect(within(await boardOf()).getByText('archived')).toBeTruthy();
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
