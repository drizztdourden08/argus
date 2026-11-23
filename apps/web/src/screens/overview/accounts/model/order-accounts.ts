/* @layer renderer-app @kind logic */
import type { Account } from '../accounts.type';

const orderAccounts = (accounts: readonly Account[]): Account[] => [
  ...accounts.filter((account) => account.active === true),
  ...accounts.filter((account) => account.active !== true),
];

export { orderAccounts };
