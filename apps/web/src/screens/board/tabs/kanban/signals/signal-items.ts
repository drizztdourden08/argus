/* @layer renderer-app @kind logic */
import type { CardData } from '../model';
import { archivedSignal } from './archived-signal';
import { conflictSignal } from './conflict-signal';
import { kindSignal } from './kind-signal';
import { needsSignal } from './needs-signal';
import { TRAIL_AFTER } from './signal.constants';
import type { CardSignal } from './signal.type';
import { trailSignal } from './trail-signal';

const present = (signals: readonly (CardSignal | null)[]): CardSignal[] => signals.filter((signal): signal is CardSignal => signal !== null);

const withTrail = (signals: CardSignal[], trail: CardSignal | null): CardSignal[] => {
  if (!trail) return signals;
  const last = signals.reduce((found, signal, index) => (TRAIL_AFTER.includes(signal.id) ? index : found), -1);
  const at = last < 0 ? signals.length : last + 1;
  return [...signals.slice(0, at), trail, ...signals.slice(at)];
};

const attentionSignals = (card: CardData, now: number): CardSignal[] =>
  present([needsSignal(card), ...card.attention.map((kind) => kindSignal(kind, card, now))]);

const signalItems = (card: CardData, now: number): CardSignal[] => [
  ...present([conflictSignal(card)]),
  ...withTrail(attentionSignals(card, now), trailSignal(card)),
  ...present([archivedSignal(card)]),
];

export { signalItems };
