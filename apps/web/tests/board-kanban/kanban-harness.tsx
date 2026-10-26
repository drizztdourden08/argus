/* @layer renderer-app @kind test */
import { TesseraProvider, ToastStack, toast } from '@drizztdourden08/tessera';
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter, useLocation } from 'react-router';
import { afterEach, expect, vi } from 'vitest';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { BoardMore, useBoardMore } from '../../src/screens/board/BoardMore';
import { KanbanPurge, KanbanTab, useTrashCount } from '../../src/screens/board/tabs/kanban';
import { createFakeSources } from '../helpers/fake-source';
import { snapshot } from '../helpers/snapshot';

type Doc = Record<string, unknown>;

interface Board {
  items?: Doc[];
  threads?: Doc[];
  entries?: Doc[];
  actions?: Doc[];
  tests?: Doc[];
  builds?: Doc[];
  log?: Doc[];
  agents?: Doc[];
}

interface Posted {
  url: string;
  body: Doc;
}

type Reply = (body: Doc) => Doc | undefined;

const SCOPE = 'alpha';
const where = { hash: '' };

const Probe = () => {
  where.hash = useLocation().hash;
  return null;
};
const Stage = () => {
  const more = useBoardMore();
  const count = useTrashCount(SCOPE);
  return (
    <>
      <BoardMore trashCount={count} onTrash={more.openTrash} onPurge={() => more.openPurge({ scope: 'thread' })} />
      <KanbanTab boardKey={SCOPE} trash={more.trashOpen} onTrashBack={more.closeTrash} onPurge={(scope) => more.openPurge({ scope })} />
      <KanbanPurge boardKey={SCOPE} request={more.purge} onClose={more.closePurge} />
    </>
  );
};

const moreItem = async (name: string | RegExp) => {
  fireEvent.click(screen.getByRole('button', { name: 'More' }));
  fireEvent.click(await screen.findByRole('menuitem', { name }));
};

const MIN = 60_000;
const HOUR = 3_600_000;
const DAY = 86_400_000;

const ago = (ms: number): string => new Date(Date.now() - ms).toISOString();

const item = (id: string, extra: Doc = {}): Doc => ({
  id,
  title: `Item ${id}`,
  description: '',
  subject: 'general',
  thread: 'worker',
  status: 'todo',
  rank: 1024,
  rev: 3,
  ac: [],
  createdAt: ago(30 * DAY),
  updatedAt: ago(MIN),
  ...extra,
});

const thread = (id: string, extra: Doc = {}): Doc => ({ id, name: id, session: id, role: 'thread', status: 'active', ...extra });

const byId = (docs: Doc[]): Record<string, object> => Object.fromEntries(docs.map((doc) => [String(doc.id), doc]));

const applied = { id: 'entry', status: 'applied', outcome: null, entry: null, seq: 1 };

const stubPosts = (reply?: Reply): Posted[] => {
  const posted: Posted[] = [];
  vi.stubGlobal('fetch', vi.fn((url: string, init?: RequestInit) => {
    const body = JSON.parse(String(init?.body ?? '{}')) as Doc;
    posted.push({ url, body });
    return Promise.resolve(new Response(JSON.stringify(reply?.(body) ?? applied), { status: 200 }));
  }));
  return posted;
};

const mountKanban = async (board: Board = {}, reply?: Reply, hash = '') => {
  vi.stubGlobal('CSS', { supports: () => false });
  const posted = stubPosts(reply);
  const fake = createFakeSources();
  const view = render(
    <MemoryRouter initialEntries={[`/p/${SCOPE}/${hash}`]}>
      <TesseraProvider overrides={{}}>
        <DataProvider streams={createScopeStreams(fake.open)}>
          <Stage />
          <Probe />
          <ToastStack />
        </DataProvider>
      </TesseraProvider>
    </MemoryRouter>,
  );
  await waitFor(() => expect(fake.sources.length).toBeGreaterThan(0));
  const feed = (coll: string, docs: Doc[]) => fake.sources.forEach((source) => source.emit('snapshot', snapshot(SCOPE, coll, byId(docs))));
  act(() => {
    feed('items', board.items ?? []);
    feed('threads', board.threads ?? [thread('worker')]);
    feed('entries', board.entries ?? []);
    feed('actions', board.actions ?? []);
    feed('tests', board.tests ?? []);
    feed('builds', (board.builds ?? []).map((build) => ({ ...build, id: build.tag })));
    feed('log', board.log ?? []);
    feed('agents', board.agents ?? []);
  });
  let seq = 98;
  const change = (coll: string, id: string, data: Doc) => {
    seq += 1;
    act(() => fake.sources.forEach((source) => source.emit('changes', { scope: SCOPE, seq, changes: [{ coll, id, op: 'set', data }] })));
  };
  return { ...view, posted, change };
};

const cleanKanban = (): void => {
  afterEach(() => {
    act(() => toast.clear());
    cleanup();
    vi.unstubAllGlobals();
    localStorage.clear();
  });
};

const columnOf = (name: string): HTMLElement => screen.getByRole('group', { name });

const boardOf = async (): Promise<HTMLElement> => screen.findByRole('group', { name: 'Work items' });

const openItem = async (id: string): Promise<HTMLElement> => {
  fireEvent.click(await screen.findByRole('button', { name: new RegExp(`^Open ${id}:`) }));
  return screen.getByRole('dialog', { name: 'Work item' });
};

const gripOf = (id: string, title = `Item ${id}`): HTMLElement => screen.getByRole('button', { name: `Move ${id} ${title}` });

export { DAY, HOUR, MIN, ago, boardOf, cleanKanban, columnOf, gripOf, item, moreItem, mountKanban, openItem, thread, where };
export type { Board, Doc, Posted };
