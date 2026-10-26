/* @layer renderer-app @kind test */
import { vi } from 'vitest';

interface ApiCall {
  path: string;
  method: string;
  body: unknown;
}

interface ApiReply {
  status?: number;
  body?: unknown;
}

type Replier = (call: ApiCall) => ApiReply | undefined;

const stubApi = (reply: Replier = () => undefined): ApiCall[] => {
  const calls: ApiCall[] = [];
  vi.stubGlobal('fetch', vi.fn((path: string, init?: RequestInit) => {
    const call: ApiCall = { path, method: init?.method ?? 'GET', body: init?.body ? JSON.parse(String(init.body)) : undefined };
    calls.push(call);
    const answer = reply(call) ?? { body: path.startsWith('/api/runs') ? { runs: [] } : {} };
    return Promise.resolve(new Response(JSON.stringify(answer.body ?? {}), { status: answer.status ?? 200 }));
  }));
  return calls;
};

export { stubApi };
export type { ApiCall };
