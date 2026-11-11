/* @layer renderer-app @kind logic */
import type { SyncStepKind } from '../../../../data/types';
import { STEP_KIND_LABEL, STEP_KINDS } from '../compare.constants';
import type { ChipLook, SyncStep } from '../compare.type';

const kindLook = (kind: SyncStepKind, steps: readonly SyncStep[]): ChipLook => {
  const mine = steps.filter((step) => step.kind === kind);
  const label = STEP_KIND_LABEL[kind];
  if (mine.length === 0) return { label: `${label} pending`, tone: 'neutral' };
  if (mine.some((step) => step.status === 'failed')) return { label: `${label} failed`, tone: 'danger' };
  if (mine.some((step) => step.status === 'running')) return { label: `${label} running`, tone: 'info' };
  return { label: `${label} done ${mine.length}`, tone: 'success' };
};

const stepKinds = (steps: readonly SyncStep[]): ChipLook[] => STEP_KINDS.map((kind) => kindLook(kind, steps));

export { stepKinds };
