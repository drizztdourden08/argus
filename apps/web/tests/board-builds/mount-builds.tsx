/* @layer renderer-app @kind test */
import { act, render, waitFor } from '@testing-library/react';
import { expect, vi } from 'vitest';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { BuildsTab } from '../../src/screens/board/tabs/builds';
import { byId } from '../board-tests/fixtures';
import { stubFetch } from '../board-tests/mount-tab';
import type { Responder } from '../board-tests/mount-tab';
import { createFakeSources, FakeSource } from '../helpers/fake-source';
import { snapshot } from '../helpers/snapshot';

type Doc = Record<string, unknown>;

interface Board {
  launchers?: Doc[];
  launches?: Doc[];
  runs?: Doc[];
  builds?: Doc[];
}

const SCOPE = 'rotp';

const stubEventSource = (): FakeSource[] => {
  const streams: FakeSource[] = [];
  vi.stubGlobal('EventSource', class extends FakeSource {
    constructor(url: string) {
      super(url);
      streams.push(this);
    }
  });
  return streams;
};

const mountBuilds = async (board: Board = {}, respond?: Responder) => {
  const calls = stubFetch(respond);
  const streams = stubEventSource();
  const fake = createFakeSources();
  const view = render(
    <DataProvider streams={createScopeStreams(fake.open)}>
      <BuildsTab boardKey={SCOPE} />
    </DataProvider>,
  );
  await waitFor(() => expect(fake.sources.length).toBeGreaterThan(0));
  const source = fake.sources[fake.sources.length - 1];
  const feed = (coll: string, docs: Doc[], key: string) => source?.emit('snapshot', snapshot(SCOPE, coll, byId(docs, key)));
  act(() => {
    feed('launchers', board.launchers ?? [], 'name');
    feed('launches', board.launches ?? [], 'name');
    feed('runs', board.runs ?? [], 'id');
    feed('builds', board.builds ?? [], 'tag');
  });
  const change = (coll: string, id: string, data: object) =>
    act(() => source?.emit('changes', { scope: SCOPE, seq: 99, changes: [{ coll, id, op: 'set', data }] }));
  const remove = (coll: string, id: string) =>
    act(() => source?.emit('changes', { scope: SCOPE, seq: 100, changes: [{ coll, id, op: 'delete' }] }));
  return { ...view, calls, change, remove, streams };
};

export { mountBuilds };
