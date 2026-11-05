/* @layer renderer-app @kind test */
import { TesseraProvider, Text, ToastStack } from '@drizztdourden08/tessera';
import { act } from '@testing-library/react';
import { MemoryRouter, useLocation } from 'react-router';
import { vi } from 'vitest';
import { BoardTabsRow } from '../../src/screens/board/BoardTabsRow';
import { TestsTab } from '../../src/screens/board/tabs/tests';
import { mountStreamed } from '../helpers/mount-streamed';
import { snapshot } from '../helpers/snapshot';
import { byId } from './fixtures';
import type { Doc } from './fixtures';

type Feed = 'tests' | 'launchers' | 'launches' | 'runs' | 'items' | 'threads' | 'entries' | 'wakes';

type Board = Partial<Record<Feed, Doc[]>> & {
  group?: string;
  at?: string;
};

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

const SCOPE = 'rotp';
const FEEDS: [Feed, string][] = [
  ['tests', 'id'], ['launchers', 'name'], ['launches', 'name'], ['runs', 'id'], ['items', 'id'], ['threads', 'id'], ['entries', 'id'], ['wakes', 'id'],
];
const POSTED = { id: 'O-1', status: 'applied', outcome: null, entry: null, seq: 1 };

const HashProbe = () => {
  const { hash, search } = useLocation();
  return (
    <>
      <Text data-testid="hash">{hash}</Text>
      <Text data-testid="search">{search}</Text>
    </>
  );
};

const GROUP_KEY = 'coord.g.tests5';

const defaultReply = (call: Call): Reply => {
  if (call.url.endsWith('/proofs') || call.url.includes('/proofs?')) return { status: 200, body: { test: 'T-1', proof: { n: 1, url: '/files/rotp/proofs/T-1-1.png', caption: 'c' } } };
  if (call.url.endsWith('/stop')) return { status: 200, body: { run: { id: 'R-1' } } };
  if (call.url.endsWith('/runs')) return { status: 200, body: { run: { id: 'R-9' }, already: false } };
  return { status: 200, body: POSTED };
};

const bodyOf = (init: RequestInit | undefined): unknown => {
  if (typeof init?.body === 'string') return JSON.parse(init.body);
  return init?.body ? 'blob' : null;
};

const stubFetch = (respond: Responder | undefined): Call[] => {
  const calls: Call[] = [];
  vi.stubGlobal('fetch', (url: string, init?: RequestInit) => {
    const call: Call = { url, method: init?.method ?? 'GET', body: bodyOf(init) };
    calls.push(call);
    const reply = respond?.(call) ?? defaultReply(call);
    return Promise.resolve(new Response(JSON.stringify(reply.body), { status: reply.status, headers: { 'Content-Type': 'application/json' } }));
  });
  return calls;
};

const mountTab = async (board: Board = {}, respond?: Responder) => {
  vi.stubGlobal('CSS', { supports: () => false });
  if (board.group !== 'default' && (board.group || localStorage.getItem(GROUP_KEY) === null)) localStorage.setItem(GROUP_KEY, board.group ?? 'item');
  const calls = stubFetch(respond);
  const { view, source, live } = await mountStreamed(
    <TesseraProvider overrides={{}}>
      <MemoryRouter initialEntries={[board.at ?? '/']}>
        <BoardTabsRow tabs={[{ id: 'tests', label: 'Tests' }]} activeTab="tests" onTabChange={() => undefined} />
        <TestsTab boardKey={SCOPE} />
        <HashProbe />
      </MemoryRouter>
      <ToastStack />
    </TesseraProvider>,
  );
  const feed = (coll: string, docs: Doc[], key = 'id') => source?.emit('snapshot', snapshot(SCOPE, coll, byId(docs, key)));
  act(() => {
    for (const [coll, key] of FEEDS) feed(coll, board[coll] ?? [], key);
    source?.emit('snapshot', snapshot(SCOPE, 'meta', { board: { name: 'RotP' } }));
  });
  const change = (coll: string, id: string, data: object) =>
    act(() => live(SCOPE)?.emit('changes', { scope: SCOPE, seq: 99, changes: [{ coll, id, op: 'set', data }] }));
  return { ...view, calls, change };
};

export { mountTab, stubFetch };
export type { Call, Doc, Responder };
