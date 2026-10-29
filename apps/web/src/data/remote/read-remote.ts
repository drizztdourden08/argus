/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { RemoteInfo } from './remote.type';

const readRemote = (): Promise<RemoteInfo> => apiRequest<RemoteInfo>('/api/remote');

export { readRemote };
