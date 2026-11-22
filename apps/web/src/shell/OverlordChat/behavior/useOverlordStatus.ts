/* @layer renderer-app @kind logic */
import { STATUS_POLL_MS, readStatus } from '../../../data/overlord';
import type { OverlordStatus } from '../../../data/overlord';
import { useRemoteRead } from '../../../data/remote';
import type { RemoteRead } from '../../../data/remote';

const useOverlordStatus = (): RemoteRead<OverlordStatus> => useRemoteRead(readStatus, STATUS_POLL_MS);

export { useOverlordStatus };
