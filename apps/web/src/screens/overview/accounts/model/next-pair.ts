/* @layer renderer-app @kind logic */
import type { AccountsStatus } from '../../../../data/types';
import type { SlotPair } from '../next.type';
import type { NextPair } from './next-steps.type';

const pickedOf = (next: AccountsStatus['next']): SlotPair => {
  const plan = next?.plan ?? (next?.source === 'owner' ? next : null);
  return { left: plan?.from ?? null, right: plan?.to ?? null };
};

const autoOf = (next: AccountsStatus['next']): SlotPair => ({ left: next?.from ?? null, right: next?.to ?? null });

const nextPair = (status: AccountsStatus | null): NextPair => {
  const next = status?.next;
  return { picked: pickedOf(next), auto: autoOf(next), when: next?.when, phase: status?.run?.phase };
};

export { nextPair };
