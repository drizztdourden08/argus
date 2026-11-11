/* @layer renderer-app @kind logic */
import type { AccountDoc } from '../../../../data/types';
import { severityOf } from './severity-of';

const countCritical = (accounts: readonly AccountDoc[]): number =>
  accounts.filter((account) => severityOf(account) === 'critical').length;

export { countCritical };
