/* @layer renderer-app @kind logic */
import type { FeedCard } from '../../feed-model';

const feedKind = (card: FeedCard): string => {
  if (card.type === 'question' || card.type === 'decision' || card.type === 'notice') return card.type;
  return card.type === 'test' || card.type === 'run' ? 'test' : 'note';
};

export { feedKind };
