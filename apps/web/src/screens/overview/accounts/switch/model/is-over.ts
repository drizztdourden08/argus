/* @layer renderer-app @kind logic */
import type { AccountsSwitch } from '../../../../../data/types';
import { FINISHED_PHASES } from '../switch.constants';

const isOver = (doc: AccountsSwitch | null, target: string): boolean =>
  doc?.to === target && doc.phase !== undefined && FINISHED_PHASES.has(doc.phase) && doc.phase !== 'idle';

export { isOver };
