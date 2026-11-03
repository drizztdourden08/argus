/* @layer renderer-app @kind logic */
import { askIds } from './ask-ids';
import { ATTENTION } from './attention.constants';
import type { AttentionFacts, AttentionKind } from './attention.type';
import { dependencyCauses } from './dependency-causes';
import { lockCauses } from './lock-causes';

const appliesOf = (facts: AttentionFacts): Record<AttentionKind, boolean> => {
  const { item, asks } = facts;
  return {
    question: askIds(item, asks, 'question').length > 0,
    todo: (facts.needs?.todos ?? 0) > 0,
    'test-failing': facts.signals.failing > 0,
    dependency: dependencyCauses(item).length > 0,
    blocked: lockCauses(item, asks).length > 0,
    'test-pending': facts.signals.pending > 0,
    decision: askIds(item, asks, 'decision').length > 0,
    quiet: facts.quietMs !== null,
    agent: facts.signals.agents > 0,
  };
};

const attentionOf = (facts: AttentionFacts): AttentionKind[] => {
  const applies = appliesOf(facts);
  return ATTENTION.filter((spec) => applies[spec.kind]).map((spec) => spec.kind);
};

export { attentionOf };
