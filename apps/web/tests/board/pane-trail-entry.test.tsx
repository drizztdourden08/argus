/* @layer renderer-app @kind test */
import { toast } from '@drizztdourden08/tessera';
import { act, cleanup, fireEvent, screen, within } from '@testing-library/react';
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
const work = {
  id: 'I-1', title: 'Item I-1', thread: 'worker', status: 'doing', rank: 1, rev: 1, createdAt: NOW, updatedAt: NOW,
  blocked: [{ kind: 'ask', id: 'Q-1', label: 'Ship it today?' }],
};
const question = { id: 'Q-1', kind: 'question', item: 'I-1', thread: 'worker', author: 'worker', status: 'open', text: 'Ship it today?', createdAt: NOW };

const trail = {
  depth: 2,
  open: 2,
  root: {
    project: 'rotp', item: 'I-1', title: 'Item I-1', type: 'task', lane: 'doing', stage: null, assignee: 'worker', edge: null, cycle: false,
    children: [{
      project: 'rotp', item: 'Q-1', title: 'Ship it today?', type: 'question', lane: 'open', stage: null, assignee: 'worker',
      edge: { kind: 'ask', id: 'Q-1', state: 'open', since: null }, cycle: false, children: [],
    }],
  },
};

const mount = async () => {
  vi.stubGlobal('CSS', { supports: () => false });
  vi.stubGlobal('fetch', vi.fn((url: string) => Promise.resolve(new Response(JSON.stringify(url.endsWith('/trail') ? trail : {})))));
  const view = renderAt('/p/rotp/#I-1/links');
  const stream = await liveStream(view.sources, 'rotp');
  const meta = { board: { project: 'rotp', name: 'Relic of the Past' } };
  act(() => seedBoard(stream, { meta, threads: { worker }, items: { 'I-1': work }, entries: { 'Q-1': question } }));
};

describe('I-77 P6 a leaf of the trail', { tags: ['standard'] }, () => {
  it('opens the entry pane when a question at the end of the trail is clicked', async () => {
    await mount();
    const drawer = await screen.findByRole('dialog', { name: 'Work item' });
    const tree = await within(drawer).findByRole('list', { name: 'Dependency trail' });
    fireEvent.click(within(tree).getByRole('button', { name: 'Open Q-1: Ship it today?' }));
    const pane = await screen.findByRole('dialog', { name: 'Entry detail' });
    expect(within(pane).getByRole('heading', { name: 'Q-1' })).toBeTruthy();
    expect(within(pane).getByText('Ship it today?')).toBeTruthy();
  });
});
