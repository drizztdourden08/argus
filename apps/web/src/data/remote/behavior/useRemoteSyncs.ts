/* @layer renderer-app @kind logic */
import { readRemote } from '../read-remote';
import { REMOTE_POLL_MS } from '../remote.constants';
import type { SyncMap } from '../remote.type';
import { useRemoteRead } from './useRemoteRead';

const NO_SYNCS: SyncMap = {};

// The last full remote sync of each project: null until the first answer, empty when none was recorded.
const useRemoteSyncs = (): SyncMap | null => {
  const { data } = useRemoteRead(readRemote, REMOTE_POLL_MS);
  return data ? (data.syncs ?? NO_SYNCS) : null;
};

export { useRemoteSyncs };
