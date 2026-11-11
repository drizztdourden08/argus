/* @layer renderer-app @kind logic */
import type { AccountsSwitch } from '../../../../../data/types';
import { FINISHED_PHASES } from '../switch.constants';

const isRunning = (doc: AccountsSwitch | null, target: string): boolean =>
  doc?.phase !== undefined && !FINISHED_PHASES.has(doc.phase) && doc.to === target;

export { isRunning };
