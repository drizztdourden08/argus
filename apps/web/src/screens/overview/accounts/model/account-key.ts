/* @layer renderer-app @kind logic */
import type { Account } from '../accounts.type';

const accountKey = (account: Account): string => account.key ?? account.id;

export { accountKey };
