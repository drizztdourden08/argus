/* @layer renderer-app @kind logic */
import type { Entry } from '../../../../data/types';
import { FEED_KIND, OWNER_ASSIGNEE } from '../../feed-model';
import type { FeedCard } from '../../feed-model';
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

const bundleTitle = (count: number, first: string): string => `${count} ${count > 1 ? 'tests' : 'test'} to run: ${first}${count > 1 ? ' ...' : ''}`;

const cardFacts = (card: FeedCard): CardFacts => {
  const kind = FEED_KIND[card.type];
  if (card.type === 'action') {
    const { id, text, thread, item, assignee } = card.data;
    return { kind, code: id, title: plainText(text), thread, item: item ?? null, at: card.at, assignee: assignee || OWNER_ASSIGNEE };
  }
  if (card.type === 'test') return testFacts(kind, card.data, card.data.title, card.at);
  if (card.type === 'bundle') {
    const [first] = card.data.tests;
    return testFacts(kind, first ?? { id: card.data.build, thread: card.data.thread }, bundleTitle(card.data.tests.length, first?.title ?? ''), card.at);
  }
  return entryFacts(kind, card.data, card.at);
};

export { cardFacts };
