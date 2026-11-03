/* @layer renderer-app @kind test */
import { ToastStack } from '@drizztdourden08/tessera';
import { act, render, waitFor } from '@testing-library/react';
import type { ReactElement } from 'react';
import { expect } from 'vitest';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { byId } from '../board-tests/fixtures';
import { createFakeSources } from '../helpers/fake-source';
import { snapshot } from '../helpers/snapshot';

interface Feed {
  coll: string;
  docs: readonly object[];
  key: string;
}

const SCOPE = 'rotp';

const mountBoard = async (ui: ReactElement, feeds: readonly Feed[]) => {
  const fake = createFakeSources();
  const view = render(
    <DataProvider streams={createScopeStreams(fake.open)}>
      {ui}
      <ToastStack />
    </DataProvider>,
  );
  await waitFor(() => expect(fake.sources.length).toBeGreaterThan(0));
  const source = fake.sources[fake.sources.length - 1];
  act(() => {
    for (const feed of feeds) source?.emit('snapshot', snapshot(SCOPE, feed.coll, byId(feed.docs, feed.key)));
  });
  return { view, source };
};

export type { Feed };
export { SCOPE, mountBoard };
