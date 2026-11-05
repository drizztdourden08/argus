/* @layer renderer-app @kind logic */
import type { Entry } from '../../../../data/types';
import { FEED_KIND, OWNER_ASSIGNEE } from '../../feed-model';
import type { FeedCard, RunCard } from '../../feed-model';
import { plainText } from '../../plain-text';
import { RESULT_TAB } from '../OwnerFeed.constants';
import type { CardFacts, TestRef } from '../OwnerFeed.type';

const entryFacts = (kind: string, entry: Entry, at: string): CardFacts => ({
  kind,
  code: entry.id,
  title: plainText(entry.text),
  thread: entry.thread ?? null,
  item: entry.item ?? null,
  at,
  assignee: kind === 'message' || kind === 'notice' ? (entry.thread ?? '') : OWNER_ASSIGNEE,
});

const sentFacts = (entry: Entry, at: string): CardFacts => ({ ...entryFacts(entry.kind, entry, at), assignee: entry.thread ?? '' });

const testFacts = (kind: string, test: TestRef, title: string, at: string): CardFacts => ({
  kind,
  code: test.id,
  title,
  thread: test.thread,
  item: test.item ?? null,
  at,
  assignee: OWNER_ASSIGNEE,
  tab: RESULT_TAB,
});

const runFacts = (kind: string, run: RunCard, at: string): CardFacts => ({
  kind,
  code: run.id,
  title: run.reported ? `Run ${run.n} of ${run.planTitle} waits for you to pass it` : `Run ${run.n} of ${run.planTitle} ready to test`,
  thread: run.thread,
  item: run.plan || null,
  at,
  assignee: OWNER_ASSIGNEE,
});

const actionAssignee = (assignee: string | null | undefined): string => (assignee === undefined || assignee === null || assignee === '' ? OWNER_ASSIGNEE : assignee);

const cardFacts = (card: FeedCard): CardFacts => {
  const kind = FEED_KIND[card.type];
  if (card.type === 'action') {
    const { id, text, thread, item, assignee } = card.data;
    return { kind, code: id, title: plainText(text), thread, item: item ?? null, at: card.at, assignee: actionAssignee(assignee) };
  }
  if (card.type === 'test') return testFacts(kind, card.data, card.data.title, card.at);
  if (card.type === 'run') return runFacts(kind, card.data, card.at);
  if (card.type === 'sent') return sentFacts(card.data, card.at);
  return entryFacts(kind, card.data, card.at);
};

export { cardFacts };
