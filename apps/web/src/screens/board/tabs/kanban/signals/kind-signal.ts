/* @layer renderer-app @kind logic */
import type { AttentionKind } from '../attention';
import type { CardData } from '../model';
import { SIGNAL_OF } from './kind-signal.constants';
import type { CardSignal } from './signal.type';

const kindSignal = (kind: AttentionKind, card: CardData, now: number): CardSignal | null => SIGNAL_OF[kind](card, now);

export { kindSignal };
