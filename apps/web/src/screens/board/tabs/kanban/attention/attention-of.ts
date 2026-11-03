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

const appliesOf = (facts: AttentionFacts): Record<AttentionKind, boolean> => ({
  question: hasAsk(facts, 'question'),
  todo: (facts.needs?.todos ?? 0) > 0,
  'test-failing': facts.signals.failing > 0,
  dependency: hasDependency(facts),
  blocked: isExternalBlock(facts),
  'test-pending': facts.signals.pending > 0,
  decision: hasAsk(facts, 'decision'),
  quiet: facts.quietMs !== null,
  agent: facts.signals.agents > 0,
});

const attentionOf = (facts: AttentionFacts): AttentionKind[] => {
  const applies = appliesOf(facts);
  return ATTENTION.filter((spec) => applies[spec.kind]).map((spec) => spec.kind);
};

export { attentionOf };
