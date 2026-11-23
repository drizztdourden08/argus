/* @layer renderer-app @kind logic */
import type { AccountsSwitch } from '../../../../../data/types';
import { CANCELABLE_PHASES, WAITING_PHASE } from '../switch.constants';

const canCancel = (doc: AccountsSwitch | null): boolean =>
  (doc?.phase !== undefined && CANCELABLE_PHASES.has(doc.phase)) || doc?.desktopPhase === WAITING_PHASE;

export { canCancel };
