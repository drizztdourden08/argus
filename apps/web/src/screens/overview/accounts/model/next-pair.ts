/* @layer renderer-app @kind logic */
import type { AccountsStatus } from '../../../../data/types';
import type { NextPair } from './next-steps.type';

const nextPair = (status: AccountsStatus | null): NextPair => ({
  from: status?.next?.from ?? null,
  to: status?.next?.to ?? null,
  when: status?.next?.when,
  phase: status?.run?.phase,
});

export { nextPair };
