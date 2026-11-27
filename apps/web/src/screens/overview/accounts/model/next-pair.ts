/* @layer renderer-app @kind logic */
import type { AccountsStatus, NextRunPhase } from '../../../../data/types';
import { FINISHED_PHASES } from '../next.constants';
import type { SlotPair } from '../next.type';
import type { NextPair } from './next-steps.type';

const pickedOf = (next: AccountsStatus['next']): SlotPair => {
  const plan = next?.plan ?? (next?.source === 'owner' ? next : null);
  return { left: plan?.from ?? null, right: plan?.to ?? null };
};

const autoOf = (next: AccountsStatus['next']): SlotPair => ({ left: next?.from ?? null, right: next?.to ?? null });

const phaseOf = (status: AccountsStatus | null, picked: SlotPair, auto: SlotPair): NextRunPhase | undefined => {
  const run = status?.run;
  if (!run?.phase || !FINISHED_PHASES.has(run.phase)) return run?.phase;
  const samePair = run.from === (picked.left ?? auto.left) && run.to === (picked.right ?? auto.right);
  return samePair ? run.phase : undefined;
};

const nextPair = (status: AccountsStatus | null): NextPair => {
  const next = status?.next;
  const picked = pickedOf(next);
  const auto = autoOf(next);
  return { picked, auto, when: next?.when, phase: phaseOf(status, picked, auto) };
};

export { nextPair };
