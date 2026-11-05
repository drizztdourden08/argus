/* @layer renderer-app @kind logic */
import type { FeedSectionsProps } from '../OwnerFeed.type';
import { isAskCard } from './ask-cards';

const splitWork = (work: FeedSectionsProps['work']) => ({
  asks: work.waiting.filter(isAskCard),
  others: work.waiting.filter((card) => !isAskCard(card)),
  todos: work.leads.filter((card) => card.type !== 'sent'),
  sent: work.leads.filter((card) => card.type === 'sent'),
});

export { splitWork };
