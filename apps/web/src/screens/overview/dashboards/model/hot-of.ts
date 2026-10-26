/* @layer renderer-app @kind logic */
import type { Card } from '../../../../data/types';
import { attentionOf } from './attention-of';

const hotOf = (card: Card): number => {
  const attention = attentionOf(card);
  return attention.needs + attention.failing + attention.blocked;
};

export { hotOf };
