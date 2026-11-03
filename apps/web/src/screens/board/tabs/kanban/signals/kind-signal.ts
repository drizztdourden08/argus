/* @layer renderer-app @kind logic */
import type { AttentionKind } from '../attention';
import type { CardData } from '../model';
import { askSignal } from './ask-signal';
import { countSignal } from './count-signal';
import { dependencySignal } from './dependency-signal';
import { lockSignal } from './lock-signal';
import { quietSignal } from './quiet-signal';
import type { CardSignal } from './signal.type';

const kindSignal = (kind: AttentionKind, card: CardData, now: number): CardSignal | null => {
  const { signals, needs } = card;
  switch (kind) {
    case 'question':
    case 'decision':
      return askSignal(kind, card);
    case 'todo':
      return countSignal(kind, needs?.todos ?? 0);
    case 'test-failing':
      return countSignal(kind, signals.failing);
    case 'test-pending':
      return countSignal(kind, signals.pending);
    case 'agent':
      return countSignal(kind, signals.agents);
    case 'dependency':
      return dependencySignal(card.item);
    case 'blocked':
      return lockSignal(card);
    case 'quiet':
      return quietSignal(card, now);
  }
};

export { kindSignal };
