/* @layer renderer-app @kind logic */
import { isWaiting } from '../../../../../data/deps';
import { depLinks } from '../model/dep-links';
import { ATTENTION } from './attention.constants';
import type { AttentionFacts, AttentionKind } from './attention.type';

const hasAsk = (facts: AttentionFacts, kind: 'question' | 'decision'): boolean => facts.asks.some((link) => link.kind === kind);

const hasDependency = (facts: AttentionFacts): boolean =>
  depLinks(facts.item).some((link) => link.state === 'blocked' || (link.side === 'waits' && isWaiting(link.state)));

const isExternalBlock = (facts: AttentionFacts): boolean =>
  Boolean(facts.item.blocker?.trim()) && !facts.asks.some((link) => link.via === 'blocker');

const APPLIES: Readonly<Record<AttentionKind, (facts: AttentionFacts) => boolean>> = {
  question: (facts) => hasAsk(facts, 'question'),
  todo: (facts) => (facts.needs?.todos ?? 0) > 0,
  'test-failing': (facts) => facts.signals.failing > 0,
  dependency: hasDependency,
  blocked: isExternalBlock,
  'test-pending': (facts) => facts.signals.pending > 0,
  decision: (facts) => hasAsk(facts, 'decision'),
  quiet: (facts) => facts.quietMs !== null,
  agent: (facts) => facts.signals.agents > 0,
};

const attentionOf = (facts: AttentionFacts): AttentionKind[] =>
  ATTENTION.filter((spec) => APPLIES[spec.kind](facts)).map((spec) => spec.kind);

export { attentionOf };
