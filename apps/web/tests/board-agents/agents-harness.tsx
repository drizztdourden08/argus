/* @layer renderer-app @kind test */
import { TesseraProvider, ToastStack } from '@drizztdourden08/tessera';
import { act, render, waitFor } from '@testing-library/react';
import { expect, vi } from 'vitest';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { AgentsTab } from '../../src/screens/board/tabs/agents';
import { createFakeSources } from '../helpers/fake-source';
import { snapshot } from '../helpers/snapshot';

const MIN = 60_000;
const DAY = 86_400_000;

const ago = (ms: number): string => new Date(Date.now() - ms).toISOString();

const agent = (extra: Record<string, unknown> = {}): Record<string, unknown> => ({
  name: 'worker',
  thread: 'lead',
  task: 'do it',
  kind: 'build',
  item: null,
  branch: null,
  worktree: null,
  status: 'working',
  note: null,
  result: null,
  model: 'sonnet',
  modelWhy: 'known cause',
  startedAt: ago(10 * MIN),
  updatedAt: ago(MIN),
  finishedAt: null,
  rev: 3,
  ...extra,
});

const live = (extra: Record<string, unknown> = {}): Record<string, unknown> => ({
  lastMessage: null,
  lastTool: null,
  activity: null,
  activitySince: null,
  lastActivityAt: ago(5000),
  turns: 0,
  toolCalls: 0,
  modelUsed: null,
  tokens: { input: 0, output: 0, cacheRead: 0 },
  stalled: false,
  liveNote: null,
  ...extra,
});

interface Posted {
  url: string;
  body: Record<string, unknown>;
}

const json = (body: unknown): Response => new Response(JSON.stringify(body), { status: 200 });

const emptyReport = { ran: true, at: null, findings: [] };

type Answer = (url: string, init?: RequestInit) => Response;

const mockFetch = (answer: Answer): void => {
  vi.stubGlobal('fetch', vi.fn((url: string, init?: RequestInit) => Promise.resolve(answer(url, init))));
};

const applied = { id: 'entry', status: 'applied', outcome: null, entry: null, seq: 1 };

const mockApi = (report: unknown = emptyReport): Posted[] => {
  const posted: Posted[] = [];
  mockFetch((url, init) => {
    if (init?.method !== 'POST') return json(report);
    posted.push({ url, body: JSON.parse(String(init.body)) as Record<string, unknown> });
    return json(applied);
  });
  return posted;
};

interface Board {
  agents?: Record<string, object>;
  threads?: Record<string, object>;
}

const renderAgents = async (board: Board) => {
  const fake = createFakeSources();
  const view = render(
    <TesseraProvider overrides={{}}>
      <DataProvider streams={createScopeStreams(fake.open)}>
        <AgentsTab boardKey="alpha" />
        <ToastStack />
      </DataProvider>
    </TesseraProvider>,
  );
  await waitFor(() => expect(fake.sources.length).toBeGreaterThan(0));
  act(() => {
    for (const source of fake.sources) {
      source.emit('snapshot', snapshot('alpha', 'agents', board.agents ?? {}));
      source.emit('snapshot', snapshot('alpha', 'threads', board.threads ?? {}));
    }
  });
  return { ...view, sources: fake.sources };
};

const rowOf = (container: HTMLElement, id: string): HTMLElement => {
  const row = container.querySelector<HTMLElement>(`[data-agent-id="${id}"]`);
  if (!row) throw new Error(`no row for ${id}`);
  return row;
};

export { agent, ago, DAY, json, live, MIN, mockApi, mockFetch, renderAgents, rowOf };
export type { Posted };
