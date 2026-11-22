/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { OverlordQueue } from './overlord.type';

const readQueue = (): Promise<OverlordQueue> => apiRequest<OverlordQueue>('/api/overlord/queue');

export { readQueue };
