/* @layer renderer-app @kind test */
import { waitFor } from '@testing-library/react';
import { expect } from 'vitest';
import type { FakeSource } from './fake-source';

const FIRST_LAZY_LOAD_MS = 5000;

const waitForStream = async (sources: readonly unknown[]): Promise<void> => {
  await waitFor(() => expect(sources.length).toBeGreaterThan(0), { timeout: FIRST_LAZY_LOAD_MS });
};

const liveStream = async (sources: readonly FakeSource[], scope: string): Promise<FakeSource> => {
  const prefix = `/api/p/${scope}/stream`;
  const find = (): FakeSource | undefined => [...sources].reverse().find((source) => !source.closed && source.url.startsWith(prefix));
  return waitFor(
    () => {
      const found = find();
      if (!found) throw new Error(`no open ${scope} stream yet`);
      return found;
    },
    { timeout: FIRST_LAZY_LOAD_MS },
  );
};

export { liveStream, waitForStream };
