/* @layer renderer-app @kind logic */
import type { BoardAction, Entry } from '../../../data/types';
import { askMode } from './ask-mode';
import type { AskKind, FeedCard, FeedInput, ManualTest, OwnerMarks } from './feed.type';
import { runCards } from './run-cards';

const askType = (entry: Entry, marks: OwnerMarks): AskKind | null => {
  if (entry.status !== 'open' || marks.acked.has(entry.id)) return null;
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

const isWaiting = (test: ManualTest): boolean =>
  test.mode === 'manual' && test.status === 'pending' && !test.build && !test.archivedAt && !test.deletedAt;

const testCards = (tests: ManualTest[]): FeedCard[] =>
  tests.filter(isWaiting).map((test): FeedCard => ({ type: 'test', at: test.updatedAt, data: test }));

const buildFeed = (input: FeedInput): FeedCard[] =>
  [
    ...entryCards(input.entries, input.marks),
    ...actionCards(input.actions, input.marks),
    ...testCards(input.tests),
    ...runCards(input.items),
  ].sort((a, b) => b.at.localeCompare(a.at));

export { buildFeed };
