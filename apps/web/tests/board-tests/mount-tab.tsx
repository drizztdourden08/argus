/* @layer renderer-app @kind test */
import { act, render, waitFor } from '@testing-library/react';
import { expect, vi } from 'vitest';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { TestsTab } from '../../src/screens/board/tabs/tests';
import { createFakeSources } from '../helpers/fake-source';
import { snapshot } from '../helpers/snapshot';
import { byId } from './fixtures';

type Doc = Record<string, unknown>;

interface Board {
  tests?: Doc[];
  builds?: Doc[];
  launchers?: Doc[];
  launches?: Doc[];
  runs?: Doc[];
  items?: Doc[];
  handed?: string | null;
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

const SCOPE = 'rotp';
const POSTED = { id: 'O-1', status: 'applied', outcome: null, entry: null, seq: 1 };

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
  const calls = stubFetch(respond);
  const fake = createFakeSources();
  const view = render(
    <DataProvider streams={createScopeStreams(fake.open)}>
      <TestsTab boardKey={SCOPE} />
    </DataProvider>,
  );
  await waitFor(() => expect(fake.sources.length).toBeGreaterThan(0));
  const source = fake.sources[fake.sources.length - 1];
  const feed = (coll: string, docs: Doc[], key = 'id') => source?.emit('snapshot', snapshot(SCOPE, coll, byId(docs, key)));
  act(() => {
    feed('tests', board.tests ?? []);
    feed('builds', board.builds ?? [], 'tag');
    feed('launchers', board.launchers ?? [], 'name');
    feed('launches', board.launches ?? [], 'name');
    feed('runs', board.runs ?? []);
    feed('items', board.items ?? []);
    source?.emit('snapshot', snapshot(SCOPE, 'meta', { board: { name: 'RotP', handedBuild: board.handed ?? null } }));
  });
  const change = (coll: string, id: string, data: object) =>
    act(() => source?.emit('changes', { scope: SCOPE, seq: 99, changes: [{ coll, id, op: 'set', data }] }));
  return { ...view, calls, change };
};

export { mountTab, stubFetch };
export type { Call, Responder };
