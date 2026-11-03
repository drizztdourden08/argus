/* @layer renderer-app @kind test */
import { act, cleanup } from '@testing-library/react';
import { MemoryRouter } from 'react-router';
import { afterEach, beforeEach, vi } from 'vitest';
import type { WorktreesMeta } from '../../src/data/worktrees';
import { WorktreesTab } from '../../src/screens/board/tabs/worktrees';
import { mountStreamed } from '../helpers/mount-streamed';
import { snapshot } from '../helpers/snapshot';
import { NOW } from './fixtures';
import type { WorktreeRow } from '../../src/screens/board/tabs/worktrees/model';

interface Board {
  docs?: WorktreeRow[];
  items?: object[];
  threads?: object[];
  meta?: WorktreesMeta | null;
}

interface Call {
  url: string;
  method: string;
  body: unknown;
}

interface Reply {
  status: number;
  body: unknown;
}

type Responder = (call: Call) => Reply | undefined;

const SCOPE = 'alpha';
const POSTED = { id: 'O-1', status: 'applied', outcome: null, entry: null, seq: 1 };

const bodyOf = (init: RequestInit | undefined): unknown => (typeof init?.body === 'string' ? JSON.parse(init.body) : null);

const JSON_HEADERS = { 'Content-Type': 'application/json' };

const stubFetch = (respond: Responder | undefined): Call[] => {
  const calls: Call[] = [];
  const answer = (url: string, init?: RequestInit): Promise<Response> => {
    const call: Call = { url, method: init?.method ?? 'GET', body: bodyOf(init) };
    calls.push(call);
    const { status, body } = respond?.(call) ?? { status: 200, body: POSTED };
    return Promise.resolve(new Response(JSON.stringify(body), { status, headers: JSON_HEADERS }));
  };
  vi.stubGlobal('fetch', answer);
  return calls;
};

const cleanAfterEach = (): void => {
  beforeEach(() => {
    vi.useFakeTimers({ toFake: ['Date'] });
    vi.setSystemTime(NOW);
  });
  afterEach(() => {
    cleanup();
    vi.useRealTimers();
    vi.unstubAllGlobals();
    window.localStorage.clear();
  });
};

const byId = (docs: WorktreeRow[]): Record<string, object> => Object.fromEntries(docs.map((doc) => [doc.id, doc]));

const rowsById = (docs: object[]): Record<string, object> => Object.fromEntries(docs.map((doc) => [(doc as { id: string }).id, doc]));

const mountTab = async (board: Board = {}, respond?: Responder) => {
  const calls = stubFetch(respond);
  const { view, source } = await mountStreamed(
    <MemoryRouter>
      <WorktreesTab boardKey={SCOPE} />
    </MemoryRouter>,
  );
  act(() => {
    source?.emit('snapshot', snapshot(SCOPE, 'worktrees', byId(board.docs ?? [])));
    source?.emit('snapshot', snapshot(SCOPE, 'items', rowsById(board.items ?? [])));
    source?.emit('snapshot', snapshot(SCOPE, 'threads', rowsById(board.threads ?? [])));
    source?.emit('snapshot', snapshot(SCOPE, 'meta', board.meta ? { worktrees: board.meta } : {}));
  });
  const change = (coll: string, id: string, data: object) =>
    act(() => source?.emit('changes', { scope: SCOPE, seq: 99, changes: [{ coll, id, op: 'set', data }] }));
  const drop = (coll: string, id: string) =>
    act(() => source?.emit('changes', { scope: SCOPE, seq: 100, changes: [{ coll, id, op: 'delete' }] }));
  return { ...view, calls, change, drop };
};

export { cleanAfterEach, mountTab };
export type { Responder };
