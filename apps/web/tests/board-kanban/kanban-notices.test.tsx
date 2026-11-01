/* @layer renderer-app @kind test */
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { MIN, ago, boardOf, cleanKanban, item, mountKanban } from './kanban-harness';

cleanKanban();

const moveOp = {
  kind: 'move',
  author: 'human',
  target: { c: 'items', id: 'I-1' },
  to: { status: 'doing' },
  prev: { thread: 'worker', status: 'todo' },
  baseRev: 3,
  createdAt: ago(MIN),
};

const conflicted = { id: 'O-1-aaaa', ...moveOp, status: 'conflict', by: 'worker', current: { status: 'review' } };

const NOTICES = 'Changes the board did not take';

describe('K3 a change the board did not take: the notices', { tags: ['standard'] }, () => {
  it('says who changed the item after the owner looked, and marks the card', async () => {
    await mountKanban({ items: [item('I-1', { status: 'review', rev: 5 })], entries: [conflicted] });
    const region = await screen.findByRole('region', { name: NOTICES });
    expect(within(region).getByText('Conflict on I-1: worker changed it after you looked. Nothing of yours was applied.')).toBeTruthy();
    expect(within(await boardOf()).getByText('conflict')).toBeTruthy();
  });

  it('keeps theirs by acknowledging the entry', async () => {
    const { posted } = await mountKanban({ items: [item('I-1', { status: 'review', rev: 5 })], entries: [conflicted] });
    fireEvent.click(await screen.findByRole('button', { name: 'Keep theirs' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'ack', author: 'human', replyTo: 'O-1-aaaa', text: '' });
  });

  it('uses mine by sending the op again with force, against what is there now, then acknowledges it', async () => {
    const { posted } = await mountKanban({ items: [item('I-1', { status: 'review', rev: 5 })], entries: [conflicted] });
    fireEvent.click(await screen.findByRole('button', { name: 'Use mine' }));
    await waitFor(() => expect(posted).toHaveLength(2));
    expect(posted[0]?.body).toMatchObject({
      kind: 'move',
      target: { c: 'items', id: 'I-1' },
      to: { status: 'doing' },
      prev: { thread: 'worker', status: 'review' },
      baseRev: 5,
      force: true,
      retryOf: 'O-1-aaaa',
    });
    expect(posted[1]?.body).toMatchObject({ kind: 'ack', replyTo: 'O-1-aaaa' });
  });

  it('says a refusal with its reason and dismisses it', async () => {
    const refused = { id: 'O-2-bbbb', kind: 'edit', author: 'human', target: { c: 'items', id: 'I-1' }, set: { title: 'x' }, status: 'refused', outcome: 'not yours', createdAt: ago(MIN) };
    const { posted } = await mountKanban({ items: [item('I-1')], entries: [refused] });
    const region = await screen.findByRole('region', { name: NOTICES });
    expect(within(region).getByText('Refused edit on I-1: not yours')).toBeTruthy();
    fireEvent.click(within(region).getByRole('button', { name: 'Dismiss' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'ack', replyTo: 'O-2-bbbb' });
  });
});

describe('K3 a change the board did not take: bulk', { tags: ['standard'] }, () => {
  it('shows one notice for each op of a bulk entry the board did not take, and acknowledges the entry with the last', async () => {
    const bulk = {
      id: 'O-3-cccc',
      kind: 'bulk',
      author: 'human',
      status: 'applied',
      createdAt: ago(MIN),
      ops: [moveOp, { ...moveOp, target: { c: 'items', id: 'I-2' } }],
      results: [{ status: 'applied' }, { status: 'conflict', by: 'lead', current: { status: 'done' } }],
    };
    const { posted } = await mountKanban({ items: [item('I-1'), item('I-2', { status: 'done' })], entries: [bulk] });
    const region = await screen.findByRole('region', { name: NOTICES });
    expect(within(region).getByText('Conflict on I-2: lead changed it after you looked. Nothing of yours was applied.')).toBeTruthy();
    expect(within(region).queryByText(/Conflict on I-1/)).toBeNull();
    fireEvent.click(within(region).getByRole('button', { name: 'Keep theirs' }));
    await waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.body).toMatchObject({ kind: 'ack', replyTo: 'O-3-cccc' });
    expect(screen.queryByRole('region', { name: NOTICES })).toBeNull();
  });
});
