/* @layer renderer-app @kind logic */
import type { RemoteSync } from '../../../../data/remote';
import { agoWords } from '../../../../format';
import { syncTone } from '../../../../shell/SyncStamp/behavior/sync-tone';
import { WORDS } from '../boards-words.constants';
import type { SyncWords } from './sync-words.type';

const syncWords = (sync: RemoteSync | null, now: number): SyncWords => {
  if (!sync) return { text: WORDS.neverSynced, tone: 'muted' };
  const ago = agoWords(sync.at, now);
  if (syncTone(sync.at, now) !== 'danger') return { text: WORDS.synced(ago), tone: 'muted' };
  return { text: WORDS.syncDue(ago), tone: 'danger' };
};

export { syncWords };
