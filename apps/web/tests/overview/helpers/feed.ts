/* @layer renderer-app @kind test */
import { act } from '@testing-library/react';
import type { FakeSource } from '../../helpers/fake-source';
import { snapshot } from '../../helpers/snapshot';
import { liveStream } from '../../helpers/wait-for-stream';

type Docs = Record<string, object>;

const PORTFOLIO = '_portfolio';

const feed = async (sources: FakeSource[], collections: Record<string, Docs>): Promise<void> => {
  const source = await liveStream(sources, PORTFOLIO);
  act(() => {
    for (const [coll, docs] of Object.entries(collections)) source.emit('snapshot', snapshot(PORTFOLIO, coll, docs));
  });
};

export { feed };
