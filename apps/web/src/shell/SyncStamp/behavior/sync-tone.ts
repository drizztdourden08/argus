/* @layer renderer-app @kind logic */
import { toMillis } from '../../../format';
import { SYNC_AMBER_MS, SYNC_DUE_MS } from '../SyncStamp.constants';
import type { SyncTone } from '../SyncStamp.type';

const syncTone = (at: unknown, now: number): SyncTone => {
  const time = toMillis(at);
  if (!Number.isFinite(time)) return 'muted';
  const age = now - time;
  if (age > SYNC_DUE_MS) return 'danger';
  return age >= SYNC_AMBER_MS ? 'warning' : 'muted';
};

export { syncTone };
