/* @layer renderer-app @kind test */
import { act, cleanup, fireEvent, screen, within } from '@testing-library/react';
import { afterEach, vi } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { seedBoard } from '../helpers/seed-board';
import { liveStream } from '../helpers/wait-for-stream';
import { toast } from '@drizztdourden08/tessera';

type Docs = Record<string, object>;

interface Board {
  threads?: Docs;
  items?: Docs;
  entries?: Docs;
  registry?: Docs;
}

const NOW = new Date().toISOString();

const thread = (id: string, over: Record<string, unknown> = {}): [string, object] => [
  id,
  { id, status: 'active', role: 'thread', session: `${id} session`, subject: 'the work', note: 'Doing **the thing**', activity: 'working', lastSeen: NOW, rev: 3, ...over },
];

const item = (id: string, over: Record<string, unknown> = {}): [string, object] => [
  id,
  { id, title: `Item ${id}`, thread: 'worker', status: 'doing', rank: 1, rev: 1, createdAt: NOW, ...over },
];

const chip = (id: string, over: Record<string, unknown> = {}): [string, object] => [
  id,
  { id, project: 'rotp', thread: id, session: `${id} session`, role: 'thread', state: 'listening', ...over },
];

const docs = (...pairs: [string, object][]): Docs => Object.fromEntries(pairs);

const cleanThreads = (): void => {
  afterEach(() => {
    act(() => toast.clear());
    cleanup();
    vi.unstubAllGlobals();
    localStorage.clear();
  });
};

const posted = (fetchMock: { mock: { calls: unknown[][] } }): Record<string, unknown>[] =>
  fetchMock.mock.calls
    .filter(([, init]) => (init as RequestInit | undefined)?.method === 'POST')
    .map(([, init]) => JSON.parse(String((init as RequestInit).body)) as Record<string, unknown>);

const mountBoard = async (board: Board, at = '/p/rotp/#agents') => {
  vi.stubGlobal('CSS', { supports: () => false });
  const applied = { id: 'O-1', status: 'applied', outcome: null, entry: null, seq: 1 };
  const report = { ran: true, at: null, findings: [] };
  const fetchMock = vi.fn((_url: string, init?: RequestInit) => Promise.resolve(new Response(JSON.stringify(init?.method === 'POST' ? applied : report))));
  vi.stubGlobal('fetch', fetchMock);
  const view = renderAt(at);
  const stream = await liveStream(view.sources, 'rotp');
  act(() => seedBoard(stream, { meta: { board: { project: 'rotp', name: 'Relic of the Past' } }, ...board }));
  return { ...view, fetchMock };
};

const drawerOf = async (): Promise<HTMLElement> => screen.findByRole('dialog', { name: 'Thread detail' });

const clickChip = async (id: string): Promise<void> => {
  const name = `Open the thread ${id}`;
  if (!screen.queryByRole('button', { name })) fireEvent.click(await screen.findByRole('button', { name: /registered/ }));
  fireEvent.click(await screen.findByRole('button', { name }));
};

const openChip = async (id: string): Promise<HTMLElement> => {
  await clickChip(id);
  return drawerOf();
};

const tool = (dialog: HTMLElement, name: string): HTMLElement => within(dialog).getByRole('button', { name });

export { chip, cleanThreads, clickChip, docs, drawerOf, item, mountBoard, openChip, posted, thread, tool };
