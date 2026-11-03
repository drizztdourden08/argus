/* @layer renderer-app @kind test */
import { TesseraProvider, ToastStack } from '@drizztdourden08/tessera';
import { act, render, waitFor } from '@testing-library/react';
import { useState } from 'react';
import { MemoryRouter } from 'react-router';
import { expect, vi } from 'vitest';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { BoardFocus } from '../../src/screens/board/BoardFocus';
import { PageRunLog } from '../../src/screens/board/run-terminal';
import { OpenRunLogContext } from '../../src/screens/board/shared';
import { KanbanTab } from '../../src/screens/board/tabs/kanban';
import { RecordPaneScope } from '../../src/shell';
import { RecordPaneHost } from '../../src/shell/records/pane';
import { byId } from '../board-tests/fixtures';
import { stubFetch } from '../board-tests/mount-tab';
import type { Responder } from '../board-tests/mount-tab';
import { createFakeSources } from '../helpers/fake-source';
import { snapshot } from '../helpers/snapshot';
import { SCOPE } from './mount-board';

interface Board {
  items?: object[];
  launchers?: object[];
  launches?: object[];
  runs?: object[];
}

const FEEDS: [string, string][] = [
  ['items', 'id'], ['threads', 'id'], ['entries', 'id'], ['actions', 'id'], ['tests', 'id'], ['builds', 'tag'], ['launchers', 'name'], ['launches', 'name'], ['runs', 'id'], ['worktrees', 'id'],
];

const Stage = () => {
  const [log, setLog] = useState<string | null>(null);
  return (
    <OpenRunLogContext.Provider value={setLog}>
      <BoardFocus boardKey={SCOPE} />
      <RecordPaneScope>
        <KanbanTab boardKey={SCOPE} trash={false} onTrashBack={() => undefined} onPurge={() => undefined} />
        <RecordPaneHost boardKey={SCOPE} />
      </RecordPaneScope>
      <PageRunLog boardKey={SCOPE} id={log} onClose={() => setLog(null)} />
    </OpenRunLogContext.Provider>
  );
};

const mountPaneBuilds = async (board: Board, hash: string, respond?: Responder) => {
  vi.stubGlobal('CSS', { supports: () => false });
  stubFetch(respond);
  const fake = createFakeSources();
  const view = render(
    <TesseraProvider overrides={{}}>
      <MemoryRouter initialEntries={[`/p/${SCOPE}/${hash}`]}>
        <DataProvider streams={createScopeStreams(fake.open)}>
          <Stage />
        </DataProvider>
      </MemoryRouter>
      <ToastStack />
    </TesseraProvider>,
  );
  await waitFor(() => expect(fake.sources.length).toBeGreaterThan(0));
  act(() => {
    for (const [coll, key] of FEEDS) {
      const docs = (board as Record<string, object[] | undefined>)[coll] ?? [];
      fake.sources.forEach((source) => source.emit('snapshot', snapshot(SCOPE, coll, byId(docs, key))));
    }
  });
  return view;
};

export { mountPaneBuilds };
