/* @layer renderer-app @kind constants */
import type { AttentionKind } from '../attention';
import type { SignalOpens } from './signal.type';

const CARD_SIGNAL_MAX = 6;

const OPEN_ITEM: SignalOpens = { kind: 'item' };

const NEEDS_SIGNAL = 'needs-you';

const TRAIL_SIGNAL = 'trail';

const TRAIL_AFTER: readonly string[] = ['dependency', 'blocked'];

const PLURAL_TEXT: Readonly<Partial<Record<AttentionKind, readonly [string, string]>>> = {
  todo: ['to-do waits for you', 'to-dos wait for you'],
  'test-failing': ['test failing', 'tests failing'],
  'test-pending': ['test to run', 'tests to run'],
};

export { CARD_SIGNAL_MAX, NEEDS_SIGNAL, OPEN_ITEM, PLURAL_TEXT, TRAIL_AFTER, TRAIL_SIGNAL };
