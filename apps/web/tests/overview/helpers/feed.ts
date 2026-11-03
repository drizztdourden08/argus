/* @layer renderer-app @kind test */
import { act } from '@testing-library/react';
import type { FakeSource } from '../../helpers/fake-source';
import { snapshot } from '../../helpers/snapshot';
import { waitForStream } from '../../helpers/wait-for-stream';

type Docs = Record<string, object>;

const PORTFOLIO = '_portfolio';

const feed = async (sources: FakeSource[], collections: Record<string, Docs>): Promise<void> => {
  await waitForStream(sources);
  const source = sources[sources.length - 1];
  act(() => {
    for (const [coll, docs] of Object.entries(collections)) source?.emit('snapshot', snapshot(PORTFOLIO, coll, docs));
  });
};

export { feed };
