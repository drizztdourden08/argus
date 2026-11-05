/* @layer renderer-app @kind logic */
import { readRemote } from '../read-remote';
import { REMOTE_POLL_MS } from '../remote.constants';
import type { SyncMap } from '../remote.type';
import { useRemoteRead } from './useRemoteRead';
import { NO_SYNCS } from './use-remote-syncs.constants';

const useRemoteSyncs = (): SyncMap | null => {
  const { data } = useRemoteRead(readRemote, REMOTE_POLL_MS);
  return data ? (data.syncs ?? NO_SYNCS) : null;
};

export { useRemoteSyncs };
