/* @layer renderer-app @kind logic */
import type { Account } from '../accounts.type';

const accountName = (accounts: readonly Account[], key: string | null | undefined): string => {
  if (!key) return 'unknown';
  const found = accounts.find((account) => account.id === key || account.key === key);
  return found?.label ?? found?.key ?? key;
};

export { accountName };
