/* @layer renderer-app @kind logic */
import type { AccountsSwitch } from '../../../../../data/types';
import { RUNNING_PHASES } from '../switch.constants';

const isActive = (doc: AccountsSwitch | null): boolean => doc?.phase !== undefined && RUNNING_PHASES.has(doc.phase);

export { isActive };
