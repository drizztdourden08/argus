/* @layer renderer-app @kind logic */
import type { Account } from '../accounts.type';
import { accountName } from './account-name';

const currentName = (accounts: readonly Account[]): string | null => {
  const current = accounts.find((account) => account.active === true);
  return current ? accountName(accounts, current.id) : null;
};

export { currentName };
