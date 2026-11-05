/* @layer renderer-app @kind test */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { getState, load, resetStore, saveAuto, saveLevel } from '../../../src/data/notify/settings-store';

const fetchMock = vi.fn();
vi.stubGlobal('fetch', fetchMock);

const AUTO = { handed: true, ask: true, assigned: true, verify: true, run: true };

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });

const serve = (settings: unknown, auto: unknown) =>
  fetchMock.mockImplementation((path: string) =>
    Promise.resolve(path === '/api/notify/auto' ? json(auto) : json(settings)),
  );

beforeEach(() => {
  resetStore();
  serve({ toast: { info: true }, sound: { info: false } }, { os: true, auto: AUTO });
});

afterEach(() => fetchMock.mockReset());

describe('notify settings store', { tags: ['critical'] }, () => {
  it('reads both routes once for many callers', async () => {
    await Promise.all([load(), load(), load()]);
    expect(fetchMock).toHaveBeenCalledTimes(2);
    expect(getState()).toMatchObject({ ready: true, os: true, auto: AUTO, toast: { info: true } });
  });

  it('sends only the partial auto patch', async () => {
    await load();
    fetchMock.mockClear();
    serve({}, { os: true, auto: { ...AUTO, ask: false } });
    expect(await saveAuto({ auto: { ask: false } })).toBeNull();
    const [path, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(path).toBe('/api/notify/auto');
    expect(JSON.parse(String(init.body))).toEqual({ auto: { ask: false } });
    expect(getState().auto.ask).toBe(false);
  });

  it('rolls back and returns the refusal when the server says no', async () => {
    await load();
    fetchMock.mockResolvedValueOnce(json({ error: { code: 'bad', message: 'unknown key' } }, 400));
    expect(await saveAuto({ auto: { run: false } })).toContain('unknown key');
    expect(getState().auto.run).toBe(true);
    fetchMock.mockResolvedValueOnce(json({ error: { code: 'bad', message: 'nope' } }, 400));
    expect(await saveLevel('info', { toast: false })).toContain('nope');
    expect(getState().toast.info).toBe(true);
  });
});
