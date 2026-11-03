/* @layer renderer-app @kind test */
import { cleanup, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { renderAt } from '../helpers/render-at';
import { stubEventSource } from '../helpers/stub-event-source';
import type { FakeSource } from '../helpers/fake-source';

let streams: FakeSource[] = [];

beforeEach(() => {
  vi.stubGlobal('CSS', { supports: () => false });
  streams = stubEventSource();
});
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const presenceStreams = () => streams.filter((stream) => stream.url.includes('/presence'));

const heldStream = async (): Promise<URL & { source: FakeSource }> => {
  await waitFor(() => expect(presenceStreams()).toHaveLength(1));
  const [source] = presenceStreams();
  return Object.assign(new URL(source?.url ?? '', 'http://argus.test'), { source: source as FakeSource });
};

describe('S2 board presence', { tags: ['standard'] }, () => {
  it('holds the presence stream open for the thread in the URL', async () => {
    renderAt('/p/rotp/?thread=coordinator');
    const held = await heldStream();
    expect(held.pathname).toBe('/api/p/rotp/presence');
    expect(held.searchParams.get('thread')).toBe('coordinator');
    expect(held.searchParams.get('tab')).toMatch(/^[a-z0-9]{1,8}$/);
    expect(held.source.closed).toBe(false);
  });

  it('closes the stream when the page goes away', async () => {
    const view = renderAt('/p/rotp/?thread=coordinator');
    const held = await heldStream();
    view.unmount();
    expect(held.source.closed).toBe(true);
  });
});

describe('K4 board presence', { tags: ['standard'] }, () => {
  it('strips characters the server would refuse from the thread', async () => {
    renderAt('/p/rotp/?thread=lead%20one%2F%3Cx%3E');
    expect((await heldStream()).searchParams.get('thread')).toBe('leadonex');
  });
});
