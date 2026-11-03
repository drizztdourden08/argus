/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { MIN, ago, cleanKanban, gripOf, item, mountKanban, thread } from './kanban-harness';

cleanKanban();

const key = (id: string, name: string): void => {
  fireEvent.keyDown(gripOf(id), { key: name });
};

const todo = [item('I-1', { rank: 1000 }), item('I-2', { rank: 2000 })];
const doing = item('I-3', { status: 'doing', rank: 1000 });

describe('K2 moving a card with the keyboard', { tags: ['standard'] }, () => {
  it('posts the move of the Node page: the status, the neighbour it lands above, what it was and the revision', async () => {
    const { posted } = await mountKanban({ items: [...todo, doing] });
    await screen.findByText('Item number 1');
    key('I-1', 'ArrowRight');
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.url).toBe('/api/p/alpha/entries');
    expect(posted[0]?.body).toMatchObject({
      kind: 'move',
      author: 'human',
      target: { c: 'items', id: 'I-1' },
      to: { status: 'doing', before: 'I-3' },
      prev: { thread: 'worker', status: 'todo' },
      baseRev: 3,
    });
    expect(String(posted[0]?.body.id)).toMatch(/^O-[a-z0-9]+-[a-z0-9]{4}$/);
    expect(await screen.findByText('Moved I-1 to doing.')).toBeTruthy();
  });

  it('reorders inside a column from the neighbour it lands next to', async () => {
    const { posted } = await mountKanban({ items: todo });
    await screen.findByText('Item number 1');
    key('I-1', 'ArrowDown');
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'move', target: { c: 'items', id: 'I-1' }, to: { after: 'I-2' }, baseRev: 3 });
    expect(await screen.findByText('Reordered I-1.')).toBeTruthy();
  });
});

describe('S1 moving a card in another order or grouping', { tags: ['standard'] }, () => {
  it('does not reorder when the order is by update: it says to switch to Manual and posts nothing', async () => {
    localStorage.setItem('coord.f', JSON.stringify({ sort: 'updated' }));
    const { posted } = await mountKanban({ items: [item('I-1', { updatedAt: ago(MIN) }), item('I-2', { updatedAt: ago(2 * MIN) })] });
    await screen.findByText('Item number 1');
    key('I-1', 'ArrowDown');
    expect(await screen.findByText('Switch the order to Manual to reorder.')).toBeTruthy();
    expect(posted).toHaveLength(0);
  });

  it('reassigns the thread when the board is grouped by thread', async () => {
    localStorage.setItem('coord.g.items', 'thread');
    const { posted } = await mountKanban({
      items: [item('I-1'), item('I-2', { thread: 'other' })],
      threads: [thread('worker'), thread('other')],
    });
    await screen.findByText('Item number 1');
    key('I-1', 'ArrowLeft');
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'move', to: { thread: 'other', before: 'I-2' }, prev: { thread: 'worker', status: 'todo' } });
    expect(await screen.findByText('Reassigned I-1 to other.')).toBeTruthy();
  });

  it('tells to group by status or thread instead of moving a card across assignee columns', async () => {
    localStorage.setItem('coord.g.items', 'assignee');
    const { posted } = await mountKanban({ items: [item('I-1'), item('I-2', { assignee: 'owner' })] });
    await screen.findByText('Item number 1');
    key('I-1', 'ArrowLeft');
    expect(await screen.findByText('Group by Status or Thread to move a card across columns.')).toBeTruthy();
    expect(posted).toHaveLength(0);
  });
});

describe('S3 what the owner is told about a move', { tags: ['standard'] }, () => {
  it('says a conflict plainly, as the Node page does', async () => {
    const reply = () => ({ id: 'x', status: 'conflict', outcome: 'a thread changed it', entry: null, seq: 1 });
    await mountKanban({ items: [item('I-1')] }, reply);
    await screen.findByText('Item number 1');
    key('I-1', 'ArrowRight');
    expect(await screen.findByText('Conflict: a thread changed it. Nothing of yours was applied (see the card).')).toBeTruthy();
  });

  it('says the dashboard is unreachable when the post does not arrive', async () => {
    await mountKanban({ items: [item('I-1')] });
    await screen.findByText('Item number 1');
    vi.stubGlobal('fetch', () => Promise.reject(new TypeError('offline')));
    key('I-1', 'ArrowRight');
    expect(await screen.findByText('Dashboard unreachable, retrying.')).toBeTruthy();
  });
});
