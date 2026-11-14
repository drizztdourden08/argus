/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { OverlordStatus } from './overlord.type';

const readStatus = (): Promise<OverlordStatus> => apiRequest<OverlordStatus>('/api/overlord/status');

export { readStatus };
