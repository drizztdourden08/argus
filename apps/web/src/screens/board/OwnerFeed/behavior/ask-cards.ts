/* @layer renderer-app @kind logic */
import type { FeedCard } from '../../feed-model';
import type { AskFeedCard } from '../OwnerFeedAsk.type';

const isAskCard = (card: FeedCard): card is AskFeedCard => card.type === 'decision' || card.type === 'question' || card.type === 'message';

export { isAskCard };
