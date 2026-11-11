/* @layer renderer-app @kind logic */
import type { AccountsSwitch } from '../../../../../data/types';
import { CANCELABLE_PHASES } from '../switch.constants';

const canCancel = (doc: AccountsSwitch | null): boolean => doc?.phase !== undefined && CANCELABLE_PHASES.has(doc.phase);

export { canCancel };
