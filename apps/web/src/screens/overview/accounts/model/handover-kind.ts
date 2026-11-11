/* @layer renderer-app @kind logic */
import type { HandoverRule } from '../../../../data/types';
import type { HandoverKind } from './handover-kind.type';

const handoverKind = (rule: HandoverRule | undefined): HandoverKind => (typeof rule === 'number' ? 'percent' : (rule ?? 'off'));

export { handoverKind };
