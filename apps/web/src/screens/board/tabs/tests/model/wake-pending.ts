/* @layer renderer-app @kind logic */
import { WAKE_WAIT_MS } from './model.constants';
import type { WakeDoc } from './model.type';

const wakePending = (wakes: readonly WakeDoc[], now: number): boolean =>
  wakes.some((wake) => wake.status === 'queued' && now - Date.parse(wake.at ?? '') < WAKE_WAIT_MS * 2);

export { wakePending };
