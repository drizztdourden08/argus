/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import type { BlockedCause } from '../../../../../data/types';
import { ATTENTION_OF, dependencyCauses } from '../attention';
import { LEAD_PREFIX } from '../../../../../shell/records/behavior/look.constants';
import { OPEN_ITEM } from './signal.constants';
import type { CardSignal } from './signal.type';

const projectOf = (cause: BlockedCause): string => (cause.who ?? '').replace(LEAD_PREFIX, '') || 'another board';

const depText = (cause: BlockedCause): string =>
  cause.kind === 'dep-waits'
    ? `waiting on ${projectOf(cause)}: ${cause.label}`
    : `request ${cause.id} of ${projectOf(cause)} is blocked: ${cause.label}`;

const dependencySignal = (item: ItemRef): CardSignal | null => {
  const causes = dependencyCauses(item);
  if (causes.length === 0) return null;
  const { icon, color } = ATTENTION_OF.dependency;
  return { id: 'dependency', icon, count: causes.length, label: causes.map(depText).join('; '), tone: color, opens: OPEN_ITEM };
};

export { dependencySignal };
