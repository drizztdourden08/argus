/* @layer renderer-app @kind logic */
import type { AccountsStatus } from '../../../../data/types';
import type { SlotPair } from '../next.type';
import type { NextPair } from './next-steps.type';

const pickedOf = (next: AccountsStatus['next']): SlotPair => {
  const plan = next?.plan ?? (next?.source === 'owner' ? next : null);
  return { left: plan?.from ?? null, right: plan?.to ?? null };
};

const nextPair = (status: AccountsStatus | null): NextPair => ({
  picked: pickedOf(status?.next),
  auto: { left: status?.next?.from ?? null, right: status?.next?.to ?? null },
  when: status?.next?.when,
  phase: status?.run?.phase,
});

export { nextPair };
