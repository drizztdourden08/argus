/* @layer renderer-app @kind logic */
import type { Card } from '../../../../data/types';
import { leadState } from './lead-state';
import type { Totals } from './model.type';

const totalsOf = (cards: readonly Card[], now: number): Totals => ({
  running: cards.filter((card) => leadState(card, now) === 'running').length,
  needs: cards.filter((card) => card.state !== 'not started').reduce((sum, card) => sum + (card.counts?.needsYou ?? 0), 0),
});

export { totalsOf };
