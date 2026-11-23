/* @layer renderer-app @kind logic */
import type { AskFeedCard } from './ask.type';

const feedKind = (card: AskFeedCard): string => (card.type === 'question' || card.type === 'decision' ? card.type : 'note');

export { feedKind };
