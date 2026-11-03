/* @layer renderer-app @kind test */
import { toast } from '@drizztdourden08/tessera';
import { act, cleanup, fireEvent, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { seedBoard } from '../helpers/seed-board';
import { liveStream } from '../helpers/wait-for-stream';

afterEach(() => {
  act(() => toast.clear());
  cleanup();
  vi.unstubAllGlobals();
  localStorage.clear();
});

const NOW = new Date().toISOString();

const worker = { id: 'worker', name: 'worker', session: 'worker', role: 'thread', status: 'active', activity: 'working', lastSeen: NOW };
const work = { id: 'I-1', title: 'Item I-1', thread: 'worker', status: 'doing', rank: 1, rev: 1, createdAt: NOW, updatedAt: NOW };
const question = {
  id: 'Q-1',
  kind: 'question',
  item: 'I-1',
  thread: 'worker',
  author: 'worker',
  status: 'open',
  text: 'Ship it today?',
  context: 'The suite is green.',
  options: ['Ship now', 'Wait'],
  createdAt: NOW,
};
const note = { id: 'F-1', kind: 'followup', replyTo: 'Q-1', item: 'I-1', thread: 'worker', author: 'worker', text: 'An earlier note', createdAt: NOW };

const mount = async (at: string) => {
  vi.stubGlobal('CSS', { supports: () => false });
  vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(new Response('{}'))));
  const view = renderAt(at);
  const stream = await liveStream(view.sources, 'rotp');
  const meta = { board: { project: 'rotp', name: 'Relic of the Past' } };
  act(() => seedBoard(stream, { meta, threads: { worker }, items: { 'I-1': work }, entries: { 'Q-1': question, 'F-1': note } }));
};

const paneOf = async (): Promise<HTMLElement> => screen.findByRole('dialog', { name: 'Entry detail' });

const expectDetail = (pane: HTMLElement) => {
  const inside = within(pane);
  expect(inside.getByRole('heading', { name: 'Q-1' })).toBeTruthy();
  expect(inside.getByText('Ship it today?')).toBeTruthy();
  expect(inside.getByText('The suite is green.')).toBeTruthy();
  expect(inside.getByRole('button', { name: /^Ship now/ })).toBeTruthy();
  expect(inside.getByText('open')).toBeTruthy();
  expect(within(inside.getByRole('group', { name: 'Reply F-1' })).getByText('An earlier note')).toBeTruthy();
  expect(inside.getByRole('button', { name: 'Open I-1' })).toBeTruthy();
};

describe('I-68 the entry pane', { tags: ['standard'] }, () => {
  it('opens from the question tag of a card, shows the entry in full, and closes with Escape', async () => {
    await mount('/p/rotp/#kanban');
    fireEvent.click(await screen.findByText('waiting on Q-1'));
    const pane = await paneOf();
    expectDetail(pane);
    fireEvent.keyDown(pane, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Entry detail' })).toBeNull());
  });

  it('opens from the id of a card of the owner feed', async () => {
    await mount('/p/rotp/#activity');
    fireEvent.click(await screen.findByRole('button', { name: 'Open Q-1' }));
    expectDetail(await paneOf());
  });

  it('opens from the id of an entry in the drawer of the item', async () => {
    await mount('/p/rotp/#I-1');
    const drawer = await screen.findByRole('dialog', { name: 'Work item' });
    const links = await within(drawer).findAllByRole('button', { name: 'Open Q-1' });
    fireEvent.click(links[0] as HTMLElement);
    expectDetail(await paneOf());
  });
});
