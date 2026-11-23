/* @layer renderer-app @kind logic */
import type { Account, CopyPair } from '../accounts.type';
import { accountName } from './account-name';

const copyLabel = (accounts: readonly Account[], pair: CopyPair): string =>
  `Copy ${accountName(accounts, pair.from.id)} onto ${accountName(accounts, pair.to.id)}`;

export { copyLabel };
