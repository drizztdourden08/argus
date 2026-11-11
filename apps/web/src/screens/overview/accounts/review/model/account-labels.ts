/* @layer renderer-app @kind logic */
import type { ReviewAccount } from '../../../../../data/types';

const accountLabels = (accounts: readonly ReviewAccount[]): Map<string, string> => new Map(accounts.map((account) => [account.key, account.label ?? account.key]));

export { accountLabels };
