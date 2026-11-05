/* @layer renderer-app @kind test */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { setProfile } from '../../src/data/posts';

afterEach(() => vi.unstubAllGlobals());

const stub = (status: string, outcome: string | null) => {
  const calls: { url: string; method?: string; body: { data: Record<string, unknown> } }[] = [];
  vi.stubGlobal('fetch', vi.fn((url: string, init?: RequestInit) => {
    calls.push({ url, method: init?.method, body: JSON.parse(String(init?.body)) });
    return Promise.resolve(new Response(JSON.stringify({ id: 'e', status, outcome, entry: null, seq: 1 }), { status: 200 }));
  }));
  return calls;
};

describe('I-111 set-profile', { tags: ['critical'] }, () => {
  it('puts a profile entry on the portfolio scope with only the fields given', async () => {
    const calls = stub('applied', null);
    expect(await setProfile({ avatar: 'initials', name: null })).toBeNull();
    expect(calls[0].method).toBe('PUT');
    expect(calls[0].url).toMatch(/\/api\/p\/_portfolio\/db\/entries\/[^/]+$/);
    expect(calls[0].body.data).toMatchObject({ kind: 'profile', author: 'human', avatar: 'initials', name: null });
    expect(calls[0].body.data).not.toHaveProperty('initials');
  });

  it('returns the reason when refused', async () => {
    stub('refused', 'initials must be two letters or digits');
    expect(await setProfile({ initials: '???' })).toBe('initials must be two letters or digits');
  });
});
