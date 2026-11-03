/* @layer renderer-app @kind logic */
import { OWNER_ASSIGNEE } from './feed.constants';
import type { FeedCard, FeedSection } from './feed.type';

const sectionOf = (card: FeedCard): FeedSection => {
  if (card.type === 'notice') return 'log';
  if (card.type === 'sent') return 'leads';
  if (card.type !== 'action') return 'waiting';
  const assignee = card.data.assignee;
  return !assignee || assignee === OWNER_ASSIGNEE ? 'waiting' : 'leads';
};

export { sectionOf };
