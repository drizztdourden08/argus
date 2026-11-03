/* @layer renderer-app @kind test */
import { vi } from 'vitest';

type Routes = Readonly<Record<string, unknown>>;

interface RemoteCall {
  path: string;
  method: string;
}

const JSON_HEADERS = { 'Content-Type': 'application/json' };

const answer = (path: string, routes: Routes): Response => {
  const known = routes[path] ?? routes[path.split('?')[0] ?? ''];
  if (known === undefined) return new Response('{}', { status: 404, headers: JSON_HEADERS });
  if (typeof known === 'string') return new Response(known, { status: 200 });
  return new Response(JSON.stringify(known), { status: 200, headers: JSON_HEADERS });
};

const stubRemote = (routes: Routes): RemoteCall[] => {
  const calls: RemoteCall[] = [];
  vi.stubGlobal('fetch', vi.fn((path: string, init?: RequestInit) => {
    calls.push({ path, method: init?.method ?? 'GET' });
    return Promise.resolve(answer(path, routes));
  }));
  return calls;
};

export { stubRemote };
export type { RemoteCall };
