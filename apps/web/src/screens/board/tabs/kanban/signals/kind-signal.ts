/* @layer renderer-app @kind logic */
import type { AttentionKind } from '../attention';
import type { CardData } from '../model';
import { askSignal } from './ask-signal';
import { countSignal } from './count-signal';
import { dependencySignal } from './dependency-signal';
import { lockSignal } from './lock-signal';
import { quietSignal } from './quiet-signal';
import type { CardSignal } from './signal.type';

const countOf = (kind: AttentionKind, card: CardData): number => {
  switch (kind) {
    case 'todo':
      return card.needs?.todos ?? 0;
    case 'test-failing':
      return card.signals.failing;
    case 'test-pending':
      return card.signals.pending;
    case 'agent':
      return card.signals.agents;
    default:
      return 0;
  }
};

const kindSignal = (kind: AttentionKind, card: CardData, now: number): CardSignal | null => {
  switch (kind) {
    case 'question':
    case 'decision':
      return askSignal(kind, card);
    case 'dependency':
      return dependencySignal(card.item);
    case 'blocked':
      return lockSignal(card);
    case 'quiet':
      return quietSignal(card, now);
    default:
      return countSignal(kind, countOf(kind, card));
  }
};

export { kindSignal };
