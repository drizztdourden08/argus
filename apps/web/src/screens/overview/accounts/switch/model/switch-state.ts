/* @layer renderer-app @kind logic */
import type { AccountsSwitch } from '../../../../../data/types';
import { CANCELABLE_PHASES, FINISHED_PHASES } from '../switch.constants';

const isRunning = (doc: AccountsSwitch | null, target: string): boolean =>
  doc?.phase !== undefined && !FINISHED_PHASES.has(doc.phase) && doc.to === target;

const canCancel = (doc: AccountsSwitch | null): boolean => doc?.phase !== undefined && CANCELABLE_PHASES.has(doc.phase);

const isOver = (doc: AccountsSwitch | null, target: string): boolean =>
  doc?.to === target && doc.phase !== undefined && FINISHED_PHASES.has(doc.phase) && doc.phase !== 'idle';

export { canCancel, isOver, isRunning };
