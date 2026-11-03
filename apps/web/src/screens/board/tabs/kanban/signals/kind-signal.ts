/* @layer renderer-app @kind logic */
import type { AttentionKind } from '../attention';
import type { CardData } from '../model';
import { askSignal } from './ask-signal';
import { countSignal } from './count-signal';
import { dependencySignal } from './dependency-signal';
import { lockSignal } from './lock-signal';
import { quietSignal } from './quiet-signal';
import type { CardSignal } from './signal.type';

type SignalMaker = (card: CardData, now: number) => CardSignal | null;

const SIGNAL_OF: Readonly<Record<AttentionKind, SignalMaker>> = {
  question: (card) => askSignal('question', card),
  decision: (card) => askSignal('decision', card),
  dependency: (card) => dependencySignal(card.item),
  blocked: (card) => lockSignal(card),
  quiet: (card, now) => quietSignal(card, now),
  todo: (card) => countSignal('todo', card.needs?.todos ?? 0),
  'test-failing': (card) => countSignal('test-failing', card.signals.failing),
  'test-pending': (card) => countSignal('test-pending', card.signals.pending),
  agent: (card) => countSignal('agent', card.signals.agents),
};

const kindSignal = (kind: AttentionKind, card: CardData, now: number): CardSignal | null => SIGNAL_OF[kind](card, now);

export { kindSignal };
