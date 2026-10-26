/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { MIN, ago, cleanKanban, gripOf, item, mountKanban, thread } from './kanban-harness';
import type { Doc, Posted } from './kanban-harness';

cleanKanban();

const key = (id: string, name: string): void => {
  fireEvent.keyDown(gripOf(id), { key: name });
};

const pressed = async (posted: Posted[], id: string, name: string): Promise<Doc | undefined> => {
  key(id, name);
  await waitFor(() => expect(posted).toHaveLength(1));
  return posted[0]?.body;
};

const todo = [item('I-1', { rank: 1000 }), item('I-2', { rank: 2000 })];
const doing = item('I-3', { status: 'doing', rank: 1000 });

describe('moving a card with the keyboard', () => {
  it('posts the move of the Node page: the status, the neighbour it lands above, what it was and the revision', async () => {
    const { posted } = await mountKanban({ items: [...todo, doing] });
    await screen.findByText('Item I-1');
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

  it('gives only the status for a column with no card', async () => {
    const { posted } = await mountKanban({ items: [item('I-1')] });
    await screen.findByText('Item I-1');
    expect((await pressed(posted, 'I-1', 'ArrowRight'))?.to).toEqual({ status: 'doing' });
  });

  it('reorders inside a column from the neighbour it lands next to', async () => {
    const { posted } = await mountKanban({ items: todo });
    await screen.findByText('Item I-1');
    key('I-1', 'ArrowDown');
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'move', target: { c: 'items', id: 'I-1' }, to: { after: 'I-2' }, baseRev: 3 });
    expect(await screen.findByText('Reordered I-1.')).toBeTruthy();
  });

  it('reorders upward from the card below', async () => {
    const { posted } = await mountKanban({ items: todo });
    await screen.findByText('Item I-2');
    key('I-2', 'ArrowUp');
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body.to).toEqual({ before: 'I-1' });
  });
});

describe('moving a card in another order or grouping', () => {
  it('does not reorder when the order is by update: it says to switch to Manual and posts nothing', async () => {
    localStorage.setItem('coord.f', JSON.stringify({ sort: 'updated' }));
    const { posted } = await mountKanban({ items: [item('I-1', { updatedAt: ago(MIN) }), item('I-2', { updatedAt: ago(2 * MIN) })] });
    await screen.findByText('Item I-1');
    key('I-1', 'ArrowDown');
    expect(await screen.findByText('Switch the order to Manual to reorder.')).toBeTruthy();
    expect(posted).toHaveLength(0);
  });

  it('moves to another column without a place when the order is by update', async () => {
    localStorage.setItem('coord.f', JSON.stringify({ sort: 'updated' }));
    const { posted } = await mountKanban({ items: [item('I-1'), doing] });
    await screen.findByText('Item I-1');
    expect((await pressed(posted, 'I-1', 'ArrowRight'))?.to).toEqual({ status: 'doing' });
  });

  it('reassigns the thread when the board is grouped by thread', async () => {
    localStorage.setItem('coord.g.items', 'thread');
    const { posted } = await mountKanban({
      items: [item('I-1'), item('I-2', { thread: 'other' })],
      threads: [thread('worker'), thread('other')],
    });
    await screen.findByText('Item I-1');
    key('I-1', 'ArrowLeft');
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'move', to: { thread: 'other', before: 'I-2' }, prev: { thread: 'worker', status: 'todo' } });
    expect(await screen.findByText('Reassigned I-1 to other.')).toBeTruthy();
  });

  it('sets the subject with an edit when the board is grouped by subject', async () => {
    localStorage.setItem('coord.g.items', 'subject');
    const { posted } = await mountKanban({ items: [item('I-1'), item('I-2', { subject: 'ui' })] });
    await screen.findByText('Item I-1');
    key('I-1', 'ArrowRight');
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({
      kind: 'edit',
      target: { c: 'items', id: 'I-1' },
      set: { subject: 'ui' },
      prev: { subject: 'general' },
      baseRev: 3,
    });
    expect(await screen.findByText('Subject of I-1 is now ui.')).toBeTruthy();
  });
});

describe('what the owner is told about a move', () => {
  it('says a conflict plainly, as the Node page does', async () => {
    const reply = () => ({ id: 'x', status: 'conflict', outcome: 'a thread changed it', entry: null, seq: 1 });
    await mountKanban({ items: [item('I-1')] }, reply);
    await screen.findByText('Item I-1');
    key('I-1', 'ArrowRight');
    expect(await screen.findByText('Conflict: a thread changed it. Nothing of yours was applied (see the card).')).toBeTruthy();
  });

  it('says a refusal with its reason', async () => {
    const reply = () => ({ id: 'x', status: 'refused', outcome: 'not yours', entry: null, seq: 1 });
    await mountKanban({ items: [item('I-1')] }, reply);
    await screen.findByText('Item I-1');
    key('I-1', 'ArrowRight');
    expect(await screen.findByText('Refused: not yours.')).toBeTruthy();
  });

  it('says the dashboard is unreachable when the post does not arrive', async () => {
    await mountKanban({ items: [item('I-1')] });
    await screen.findByText('Item I-1');
    vi.stubGlobal('fetch', () => Promise.reject(new TypeError('offline')));
    key('I-1', 'ArrowRight');
    expect(await screen.findByText('Dashboard unreachable, retrying.')).toBeTruthy();
  });
});
