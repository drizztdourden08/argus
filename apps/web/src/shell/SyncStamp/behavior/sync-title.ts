/* @layer renderer-app @kind logic */
import type { RemoteSync } from '../../../data/remote';
import { NEVER_SYNCED } from '../SyncStamp.constants';

const syncTitle = (sync: RemoteSync | null): string => {
  if (!sync) return NEVER_SYNCED;
  const who = sync.by ? ` by ${sync.by}` : '';
  return sync.note ? `Recorded${who}: ${sync.note}` : `Recorded${who}`;
};

export { syncTitle };
