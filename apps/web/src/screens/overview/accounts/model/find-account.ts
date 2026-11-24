/* @layer renderer-app @kind logic */
import type { Account } from '../accounts.type';

const findAccount = (accounts: readonly Account[], key: string | null | undefined): Account | null =>
  (key ? accounts.find((account) => account.id === key || account.key === key) : undefined) ?? null;

export { findAccount };
