/* @layer renderer-app @kind logic */
import { FEED_KIND, OWNER_ASSIGNEE } from '../../feed-model';
import { plainText } from '../../plain-text';
import type { CardFacts } from '../OwnerFeed.type';
import type { AskFeedCard } from './ask.type';

const cardFacts = (card: AskFeedCard): CardFacts => {
  const entry = card.data;
  return {
    kind: FEED_KIND[card.type],
    code: entry.id,
    title: plainText(entry.text),
    thread: entry.thread ?? null,
    item: entry.item ?? null,
    at: card.at,
    assignee: card.type === 'message' ? (entry.thread ?? '') : OWNER_ASSIGNEE,
  };
};

export { cardFacts };
