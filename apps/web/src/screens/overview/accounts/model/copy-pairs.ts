/* @layer renderer-app @kind logic */
import type { Account, CopyPair } from '../accounts.type';

const copyPairs = (accounts: readonly Account[]): CopyPair[] =>
  accounts.flatMap((from) => accounts.filter((to) => to.id !== from.id).map((to) => ({ from, to })));

export { copyPairs };
