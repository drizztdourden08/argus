/* @layer renderer-app @kind logic */
import type { Card } from '../../../../data/types';
import { minutesSince } from './minutes-since';
import { STALE_MINUTES } from './model.constants';
import type { LeadState } from './model.type';

const isBusy = (card: Card): boolean =>
  card.coordinator?.activity === 'working' || (card.counts?.working ?? 0) > 0 || (card.counts?.agents ?? 0) > 0;

const leadState = (card: Card, now: number): LeadState => {
  if (card.state === 'not started') return 'not started';
  if (!card.coordinator) return (card.counts?.paused ?? 0) > 0 ? 'paused' : 'idle';
  if (minutesSince(card.coordinator.lastSeen, now) > STALE_MINUTES) return 'stale';
  return isBusy(card) ? 'running' : 'idle';
};

export { leadState };
