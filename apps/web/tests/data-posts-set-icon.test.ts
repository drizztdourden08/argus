/* @layer renderer-app @kind test */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { setIcon } from '../src/data/posts';

afterEach(() => vi.unstubAllGlobals());

describe('I-105 set-icon', { tags: ['critical'] }, () => {
  it('posts the owner entry with the icon string or null', async () => {
    const sent: unknown[] = [];
    vi.stubGlobal('fetch', vi.fn((_url: string, init?: RequestInit) => {
      sent.push(JSON.parse(String(init?.body)));
      return Promise.resolve(new Response(JSON.stringify({ id: 'e', status: 'applied', outcome: null, entry: null, seq: 1 }), { status: 200 }));
    }));
    await setIcon('alpha', 'icon:code');
    await setIcon('alpha', null);
    expect(sent[0]).toMatchObject({ kind: 'icon', icon: 'icon:code', author: 'human' });
    expect(sent[1]).toMatchObject({ kind: 'icon', icon: null });
  });
});
