/* @layer renderer-app @kind test */
import { toast } from '@drizztdourden08/tessera';
import { act, cleanup } from '@testing-library/react';
import { afterEach, vi } from 'vitest';
import { renderAt } from './render-at';
import { seedBoard } from './seed-board';
import { liveStream } from './wait-for-stream';

interface EntryBoard {
  at: string;
  work: object;
  entries: Record<string, object>;
  body?: (url: string) => unknown;
}

const NOW = new Date().toISOString();

const worker = { id: 'worker', name: 'worker', session: 'worker', role: 'thread', status: 'active', activity: 'working', lastSeen: NOW };

const entryWork = (extra: object = {}): object => ({
  id: 'I-1', title: 'Item I-1', thread: 'worker', status: 'doing', rank: 1, rev: 1, createdAt: NOW, updatedAt: NOW, ...extra,
});

const cleanEntryBoard = (): void => {
  afterEach(() => {
    act(() => toast.clear());
    cleanup();
    vi.unstubAllGlobals();
    localStorage.clear();
  });
};

const mountEntryBoard = async (board: EntryBoard): Promise<void> => {
  const body = board.body ?? (() => ({}));
  vi.stubGlobal('CSS', { supports: () => false });
  vi.stubGlobal('fetch', vi.fn((url: string) => Promise.resolve(new Response(JSON.stringify(body(url))))));
  const view = renderAt(board.at);
  const stream = await liveStream(view.sources, 'rotp');
  const meta = { board: { project: 'rotp', name: 'Relic of the Past' } };
  act(() => seedBoard(stream, { meta, threads: { worker }, items: { 'I-1': board.work }, entries: board.entries }));
};

export { NOW, cleanEntryBoard, entryWork, mountEntryBoard, worker };
export type { EntryBoard };
