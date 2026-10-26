/* @layer renderer-app @kind logic */
import type { ProcDoc } from '../../../../data/types';

const needsLook = (proc: ProcDoc): boolean => typeof proc.status === 'string' && proc.status !== 'ok';

export { needsLook };
