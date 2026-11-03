/* @layer renderer-app @kind test */
import { ToastStack } from '@drizztdourden08/tessera';
import { act, render, waitFor } from '@testing-library/react';
import { expect } from 'vitest';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import type { PaneData } from '../../src/screens/board/tabs/kanban/pane/pane.type';
import { ServedTab } from '../../src/screens/board/tabs/kanban/pane/tabs/ServedTab';
import { byId } from '../board-tests/fixtures';
import { stubFetch } from '../board-tests/mount-tab';
import { createFakeSources } from '../helpers/fake-source';
import { snapshot } from '../helpers/snapshot';
import { stubEventSource } from '../helpers/stub-event-source';

type Doc = Record<string, unknown>;

interface Board {
  launchers?: Doc[];
  launches?: Doc[];
  worktrees?: object[];
}

const SCOPE = 'rotp';

const mountServed = async (itemId: string, board: Board = {}) => {
  stubFetch();
  stubEventSource();
  const fake = createFakeSources();
  const data = { boardKey: SCOPE, item: { id: itemId } } as unknown as PaneData;
  const view = render(
    <DataProvider streams={createScopeStreams(fake.open)}>
      <ServedTab data={data} />
      <ToastStack />
    </DataProvider>,
  );
  await waitFor(() => expect(fake.sources.length).toBeGreaterThan(0));
  const source = fake.sources[fake.sources.length - 1];
  const feed = (coll: string, docs: Doc[], key: string) => source?.emit('snapshot', snapshot(SCOPE, coll, byId(docs, key)));
  act(() => {
    feed('launchers', board.launchers ?? [], 'name');
    feed('launches', board.launches ?? [], 'name');
    feed('runs', [], 'id');
    feed('builds', [], 'tag');
    feed('worktrees', (board.worktrees ?? []) as Doc[], 'id');
  });
  return view;
};

export { mountServed };
