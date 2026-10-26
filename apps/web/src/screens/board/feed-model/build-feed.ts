/* @layer renderer-app @kind logic */
import type { BoardAction, Entry } from '../../../data/types';
import { askMode } from './ask-mode';
import type { AskKind, BundleGroup, FeedCard, FeedInput, ManualTest, OwnerMarks } from './feed.type';

const askType = (entry: Entry, marks: OwnerMarks): AskKind | null => {
  if (entry.status !== 'open' || marks.acked.has(entry.id)) return null;
  if (entry.kind === 'notice') return 'notice';
  if (marks.answered.has(entry.id)) return null;
  if (entry.kind === 'message') return 'message';
  if (entry.kind !== 'question') return null;
  return askMode(entry) === 'decide' ? 'decision' : 'question';
};

const entryCards = (entries: Entry[], marks: OwnerMarks): FeedCard[] =>
  entries.flatMap((entry) => {
    const type = askType(entry, marks);
    return type ? [{ type, at: entry.createdAt, data: entry }] : [];
  });

const actionCards = (actions: BoardAction[], marks: OwnerMarks): FeedCard[] =>
  actions
    .filter((action) => action.status === 'pending' && !marks.done.has(action.id))
    .map((action) => ({ type: 'action', at: action.createdAt, data: action }));

const byRankThenId = (a: ManualTest, b: ManualTest): number =>
  (a.rank ?? 0) - (b.rank ?? 0) || a.id.localeCompare(b.id, undefined, { numeric: true });

const latest = (tests: ManualTest[]): string => tests.reduce((at, test) => (test.updatedAt > at ? test.updatedAt : at), '');

const groupByBuild = (tests: ManualTest[]): Map<string, ManualTest[]> => {
  const groups = new Map<string, ManualTest[]>();
  for (const test of tests) groups.set(test.build ?? '', [...(groups.get(test.build ?? '') ?? []), test]);
  return groups;
};

const bundleOf = (build: string, tests: ManualTest[]): FeedCard => {
  const sorted = [...tests].sort(byRankThenId);
  const group: BundleGroup = { id: build, build, thread: sorted[0]?.thread ?? '', tests: sorted };
  return { type: 'bundle', at: latest(sorted), data: group };
};

const isWaiting = (test: ManualTest, closed: ReadonlySet<string>): boolean =>
  test.mode === 'manual' && test.status === 'pending' && !test.archivedAt && !test.deletedAt && !(test.build && closed.has(test.build));

const testCards = (tests: ManualTest[], closed: ReadonlySet<string>): FeedCard[] => {
  const groups = groupByBuild(tests.filter((test) => isWaiting(test, closed)));
  const loose = (groups.get('') ?? []).map((test): FeedCard => ({ type: 'test', at: test.updatedAt, data: test }));
  const bundles = [...groups].filter(([build]) => build !== '').map(([build, list]) => bundleOf(build, list));
  return [...loose, ...bundles];
};

const buildFeed = (input: FeedInput): FeedCard[] =>
  [
    ...entryCards(input.entries, input.marks),
    ...actionCards(input.actions, input.marks),
    ...testCards(input.tests, input.closed),
  ].sort((a, b) => b.at.localeCompare(a.at));

export { buildFeed };
