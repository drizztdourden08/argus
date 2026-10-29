/* @layer renderer-app @kind test */
import { act, render, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router';
import { expect } from 'vitest';
import { DataProvider } from '../../../src/data/react';
import { createScopeStreams } from '../../../src/data/stream';
import { ChangesTab } from '../../../src/screens/board/tabs/changes';
import { byId } from '../../board-tests/fixtures';
import { createFakeSources } from '../../helpers/fake-source';
import { snapshot } from '../../helpers/snapshot';
import { stubRemote } from './stub-remote';
import type { RemoteCall } from './stub-remote';

type Doc = Record<string, unknown>;

const SCOPE = 'rotp';

const mountChanges = async (prs: Doc[], routes: Record<string, unknown> = {}) => {
  const calls: RemoteCall[] = stubRemote(routes);
  const fake = createFakeSources();
  const view = render(
    <DataProvider streams={createScopeStreams(fake.open)}>
      <MemoryRouter>
        <ChangesTab boardKey={SCOPE} />
      </MemoryRouter>
    </DataProvider>,
  );
  await waitFor(() => expect(fake.sources.length).toBeGreaterThan(0));
  const source = fake.sources[fake.sources.length - 1];
  act(() => source?.emit('snapshot', snapshot(SCOPE, 'prs', byId(prs))));
  const change = (id: string, data: object) =>
    act(() => source?.emit('changes', { scope: SCOPE, seq: 99, changes: [{ coll: 'prs', id, op: 'set', data }] }));
  return { ...view, calls, change };
};

export { mountChanges };
