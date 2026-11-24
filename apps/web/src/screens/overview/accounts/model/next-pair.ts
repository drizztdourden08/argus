/* @layer renderer-app @kind logic */
import type { AccountsStatus } from '../../../../data/types';
import type { NextPair } from './next-steps.type';

type Next = AccountsStatus['next'];

const pairOf = (next: Next | undefined): Omit<NextPair, 'phase'> => ({
  from: next?.from ?? null,
  to: next?.to ?? null,
  when: next?.when,
});

const nextPair = (status: AccountsStatus | null): NextPair => ({
  ...pairOf(status?.next),
  phase: status?.run?.phase,
});

export { nextPair };
