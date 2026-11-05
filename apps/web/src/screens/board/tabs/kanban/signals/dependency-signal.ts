/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import type { BlockedCause } from '../../../../../data/types';
import { ATTENTION_OF, dependencyCauses } from '../attention';
import { LEAD_PREFIX } from '../../../../../shell/records/behavior/look.constants';
import { OPEN_ITEM } from './signal.constants';
import type { CardSignal } from './signal.type';
import { KEY_BRANDS } from '../../../../../avatar';
import { whoAvatar } from './who-avatar';

const projectOf = (cause: BlockedCause): string => (cause.who ?? '').replace(LEAD_PREFIX, '') || 'another board';

const depText = (cause: BlockedCause): string =>
  cause.kind === 'dep-waits'
    ? `waiting on ${projectOf(cause)}: ${cause.label}`
    : `request ${cause.id} of ${projectOf(cause)} is blocked: ${cause.label}`;

const byTarget = (causes: readonly BlockedCause[]): Map<string, BlockedCause[]> => {
  const groups = new Map<string, BlockedCause[]>();
  for (const cause of causes) groups.set(projectOf(cause), [...(groups.get(projectOf(cause)) ?? []), cause]);
  return groups;
};

const dependencySignals = (item: ItemRef): CardSignal[] => {
  const { icon, color } = ATTENTION_OF.dependency;
  return [...byTarget(dependencyCauses(item))].map(([target, causes]) => {
    const who = causes[0]?.who;
    const signal: CardSignal = { id: `dependency-${target}`, icon, count: causes.length, label: causes.map(depText).join('; '), tone: color, opens: OPEN_ITEM };
    const brand = KEY_BRANDS[target];
    if (brand && who?.startsWith(LEAD_PREFIX)) return { ...signal, brand };
    return who ? { ...signal, avatar: whoAvatar(who) } : signal;
  });
};

export { dependencySignals };
