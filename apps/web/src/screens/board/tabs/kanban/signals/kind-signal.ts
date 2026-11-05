/* @layer renderer-app @kind logic */
import type { AttentionKind } from '../attention';
import type { CardData } from '../model';
import { SIGNAL_OF } from './kind-signal.constants';
import type { CardSignal } from './signal.type';

const kindSignals = (kind: AttentionKind, card: CardData, now: number): CardSignal[] => {
  const made = SIGNAL_OF[kind](card, now);
  if (made === null) return [];
  return Array.isArray(made) ? made : [made];
};

export { kindSignals };
