/* @layer renderer-app @kind logic */
import type { AccountsRun } from '../../../../data/types';
import { ACTIVE_PHASES } from '../compare.constants';

const isRunActive = (run: AccountsRun | null): run is AccountsRun => run?.phase !== undefined && ACTIVE_PHASES.has(run.phase);

export { isRunActive };
