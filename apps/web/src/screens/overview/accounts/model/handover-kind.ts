/* @layer renderer-app @kind logic */
import type { HandoverRule } from '../../../../data/types';
import { DEFAULT_HANDOVER_PERCENT } from '../accounts.constants';

type HandoverKind = 'off' | 'auto' | 'percent';

const handoverKind = (rule: HandoverRule | undefined): HandoverKind => (typeof rule === 'number' ? 'percent' : (rule ?? 'off'));

const handoverRule = (kind: HandoverKind, was: HandoverRule | undefined): HandoverRule => {
  if (kind !== 'percent') return kind;
  return typeof was === 'number' ? was : DEFAULT_HANDOVER_PERCENT;
};

export { handoverKind, handoverRule };
export type { HandoverKind };
