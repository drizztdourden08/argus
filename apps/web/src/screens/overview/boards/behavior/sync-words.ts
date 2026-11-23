/* @layer renderer-app @kind logic */
import type { RemoteSync } from '../../../../data/remote';
import { formatAgo, toMillis } from '../../../../format';
import { syncTone } from '../../../../shell/SyncStamp/behavior/sync-tone';
import { MS_PER_DAY } from '../boards.constants';

interface SyncWords {
  text: string;
  tone: 'muted' | 'danger';
}

const syncWords = (sync: RemoteSync | null, now: number): SyncWords => {
  if (!sync) return { text: 'Never synced', tone: 'muted' };
  if (syncTone(sync.at, now) !== 'danger') return { text: `Synced ${formatAgo(sync.at, now)} ago`, tone: 'muted' };
  const days = Math.round((now - toMillis(sync.at)) / MS_PER_DAY);
  return { text: `Sync due, ${days} ${days === 1 ? 'day' : 'days'}`, tone: 'danger' };
};

export { syncWords };
