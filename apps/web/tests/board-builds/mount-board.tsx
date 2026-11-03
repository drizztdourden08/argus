/* @layer renderer-app @kind test */
import { ToastStack } from '@drizztdourden08/tessera';
import { act } from '@testing-library/react';
import type { ReactElement } from 'react';
import { MemoryRouter } from 'react-router';
import { byId } from '../board-tests/fixtures';
import { mountStreamed } from '../helpers/mount-streamed';
import { snapshot } from '../helpers/snapshot';

interface Feed {
  coll: string;
  docs: readonly object[];
  key: string;
}

const SCOPE = 'rotp';

const mountBoard = async (ui: ReactElement, feeds: readonly Feed[]) => {
  const { view, source, live } = await mountStreamed(
    <>
      <MemoryRouter>{ui}</MemoryRouter>
      <ToastStack />
    </>,
  );
  act(() => {
    for (const feed of feeds) source?.emit('snapshot', snapshot(SCOPE, feed.coll, byId(feed.docs, feed.key)));
  });
  return { view, source, live };
};

export type { Feed };
export { SCOPE, mountBoard };
