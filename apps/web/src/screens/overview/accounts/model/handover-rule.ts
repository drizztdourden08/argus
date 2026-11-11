/* @layer renderer-app @kind logic */
import type { HandoverRule } from '../../../../data/types';
import { DEFAULT_HANDOVER_PERCENT } from '../accounts.constants';
import type { HandoverKind } from './handover-kind.type';

const handoverRule = (kind: HandoverKind, was: HandoverRule | undefined): HandoverRule => {
  if (kind !== 'percent') return kind;
  return typeof was === 'number' ? was : DEFAULT_HANDOVER_PERCENT;
};

export { handoverRule };
