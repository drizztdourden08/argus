/* @layer renderer-app @kind constants */
import type { AttentionKind } from '../attention';
import { askSignal } from './ask-signal';
import { countSignal } from './count-signal';
import { dependencySignals } from './dependency-signal';
import type { SignalMaker } from './kind-signal.type';
import { lockSignal } from './lock-signal';
import { quietSignal } from './quiet-signal';
import { whoAvatar } from './who-avatar';

const SIGNAL_OF: Readonly<Record<AttentionKind, SignalMaker>> = {
  question: (card) => askSignal('question', card),
  decision: (card) => askSignal('decision', card),
  dependency: (card) => dependencySignals(card.item),
  blocked: (card) => lockSignal(card),
  quiet: (card, now) => quietSignal(card, now),
  todo: (card) => countSignal('todo', card.needs?.todos ?? 0, whoAvatar('owner')),
  'test-failing': (card) => countSignal('test-failing', card.signals.failing),
  'test-pending': (card) => countSignal('test-pending', card.signals.pending),
};

export { SIGNAL_OF };
