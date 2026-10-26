/* @layer renderer-app @kind test */
import { act, waitFor } from '@testing-library/react';
import { expect } from 'vitest';
import type { FakeSource } from '../../helpers/fake-source';
import { snapshot } from '../../helpers/snapshot';

type Docs = Record<string, object>;

const PORTFOLIO = '_portfolio';

const feed = async (sources: FakeSource[], collections: Record<string, Docs>): Promise<void> => {
  await waitFor(() => expect(sources.length).toBeGreaterThan(0));
  const source = sources[sources.length - 1];
  act(() => {
    for (const [coll, docs] of Object.entries(collections)) source?.emit('snapshot', snapshot(PORTFOLIO, coll, docs));
  });
};

export { feed };
