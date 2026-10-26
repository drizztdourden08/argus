/* @layer renderer-app @kind logic */
import type { Card } from '../../../../data/types';
import { ATTENTION_DECISION } from './model.constants';
import type { Attention } from './model.type';

const attentionOf = (card: Card): Attention => ({
  needs: card.counts?.needsYou ?? 0,
  failing: card.counts?.testsFailing ?? 0,
  blocked: card.counts?.blocked ?? 0,
  decisions: (card.attention ?? []).filter((line) => ATTENTION_DECISION.test(line)).length,
});

export { attentionOf };
