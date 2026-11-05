/* @layer renderer-app @kind logic */
import type { FeedSectionsProps } from '../OwnerFeed.type';
import { isAskCard } from './ask-cards';

const feedTotal = (work: FeedSectionsProps['work']): number => work.waiting.length + work.leads.length + work.notices.length + work.lines.length;

const splitWork = (work: FeedSectionsProps['work']) => ({
  asks: work.waiting.filter(isAskCard),
  others: work.waiting.filter((card) => !isAskCard(card)),
  todos: work.leads.filter((card) => card.type !== 'sent'),
  sent: work.leads.filter((card) => card.type === 'sent'),
});

export { feedTotal, splitWork };
