/* @layer renderer-app @kind test */
import { TesseraProvider, ToastStack, toast } from '@drizztdourden08/tessera';
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter, Route, Routes, useLocation } from 'react-router';
import { afterEach, beforeEach, expect, vi } from 'vitest';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { BoardMore, useBoardMore } from '../../src/screens/board/BoardMore';
import { BoardFocus } from '../../src/screens/board/BoardFocus';
import { KanbanTab } from '../../src/screens/board/tabs/kanban';
import { RecordPaneScope } from '../../src/shell';
import { RecordPaneHost } from '../../src/shell/records/pane';
import { useTrashCount } from '../../src/screens/board/tabs/kanban/behavior/useTrashCount';
import { KanbanPurge } from '../../src/screens/board/tabs/kanban/sub-components/KanbanPurge';
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
  feeds?: { coll: string; key: string; docs: Doc[] }[];
}

interface Posted {
  url: string;
  body: Doc;
}

type Reply = (body: Doc, url: string) => Doc | undefined;

const SCOPE = 'alpha';
const where = { hash: '', search: '' };

const paneOf = (): { open: string | null; pane: string | null } => {
  const params = new URLSearchParams(where.search);
  return { open: params.get('open'), pane: params.get('pane') };
};

const Probe = () => {
  const location = useLocation();
  where.hash = location.hash;
  where.search = location.search;
  return null;
};
const Stage = () => {
  const more = useBoardMore();
  const count = useTrashCount(SCOPE);
  return (
    <RecordPaneScope>
      <BoardMore boardKey={SCOPE} trashCount={count} onTrash={more.openTrash} onPurge={() => more.openPurge({ scope: 'thread' })} />
      <BoardFocus boardKey={SCOPE} />
      <KanbanTab boardKey={SCOPE} trash={more.trashOpen} onTrashBack={more.closeTrash} onPurge={(scope) => more.openPurge({ scope })} />
      <KanbanPurge boardKey={SCOPE} request={more.purge} onClose={more.closePurge} />
      <RecordPaneHost boardKey={SCOPE} />
    </RecordPaneScope>
  );
};

const moreItem = async (name: string | RegExp) => {
  fireEvent.click(screen.getByRole('button', { name: 'More' }));
  fireEvent.click(await screen.findByRole('menuitem', { name }));
};

const MIN = 60_000;
const HOUR = 3_600_000;
const DAY = 86_400_000;

const CLOCK = new Date('2026-10-26T12:00:00.000Z');

const ago = (ms: number): string => new Date(CLOCK.getTime() - ms).toISOString();

const item = (id: string, extra: Doc = {}): Doc => ({
  id,
  title: `Item number ${id.slice(2)}`,
  description: '',
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
    return Promise.resolve(new Response(JSON.stringify(reply?.(body, url) ?? applied), { status: 200 }));
  }));
  return posted;
};

type Feed = (coll: string, docs: Doc[]) => void;

const PLAIN_FEEDS = ['items', 'entries', 'actions', 'tests', 'log', 'agents'] as const;

const seedFeeds = (board: Board, feed: Feed): void => {
  for (const coll of PLAIN_FEEDS) feed(coll, board[coll] ?? []);
  feed('threads', board.threads ?? [thread('worker')]);
  feed('builds', (board.builds ?? []).map((build) => ({ ...build, id: build.tag })));
  for (const extra of board.feeds ?? []) feed(extra.coll, extra.docs.map((doc) => ({ ...doc, id: doc[extra.key] })));
  feed('meta', [{ id: 'board', project: SCOPE, name: 'Alpha' }]);
};

const mountKanban = async (board: Board = {}, reply?: Reply, hash = '') => {
  vi.stubGlobal('CSS', { supports: () => false });
  const posted = stubPosts(reply);
  const fake = createFakeSources();
  const view = render(
    <MemoryRouter initialEntries={[`/p/${SCOPE}/${hash}`]}>
      <TesseraProvider overrides={{}}>
        <DataProvider streams={createScopeStreams(fake.open)}>
          <Routes>
            <Route path="/p/:key/*" element={<Stage />} />
          </Routes>
          <Probe />
          <ToastStack />
        </DataProvider>
      </TesseraProvider>
    </MemoryRouter>,
  );
  await waitFor(() => expect(fake.sources.length).toBeGreaterThan(0));
  const feed: Feed = (coll, docs) => fake.sources.forEach((source) => source.emit('snapshot', snapshot(SCOPE, coll, byId(docs))));
  act(() => seedFeeds(board, feed));
  let seq = 98;
  const change = (coll: string, id: string, data: Doc) => {
    seq += 1;
    act(() => fake.sources.forEach((source) => source.emit('changes', { scope: SCOPE, seq, changes: [{ coll, id, op: 'set', data }] })));
  };
  return { ...view, posted, change };
};

const cleanKanban = (): void => {
  beforeEach(() => {
    vi.setSystemTime(CLOCK);
  });
  afterEach(() => {
    act(() => toast.clear());
    cleanup();
    vi.unstubAllGlobals();
    vi.useRealTimers();
    localStorage.clear();
  });
};

const columnOf = (name: string): HTMLElement => screen.getByRole('group', { name });

const boardOf = async (): Promise<HTMLElement> => screen.findByRole('group', { name: 'Work items' });

const numberOf = (id: string): string => id.replace(/^I-/, '');

const openItem = async (id: string): Promise<HTMLElement> => {
  fireEvent.click(await screen.findByRole('button', { name: new RegExp(`^Open [A-Z]{1,2}-${numberOf(id)}:`) }));
  return screen.getByRole('dialog', { name: 'Work item' });
};

const gripOf = (id: string, title = `Item number ${numberOf(id)}`): HTMLElement => screen.getByRole('button', { name: new RegExp(`^Move [A-Z]{1,2}-${numberOf(id)} ${title}$`) });

const cardOf = (id: string): HTMLElement => {
  const card = document.querySelector(`[data-item="${id}"]`)?.closest('.kanban__card');
  if (!(card instanceof HTMLElement)) throw new Error(`no card for ${id}`);
  return card;
};

export { DAY, HOUR, MIN, ago, boardOf, cardOf, cleanKanban, columnOf, gripOf, item, moreItem, mountKanban, openItem, paneOf, thread, where };
export type { Board, Doc, Posted };
