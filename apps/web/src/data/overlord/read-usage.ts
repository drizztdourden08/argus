/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { OverlordUsage } from './overlord.type';

const readUsage = (): Promise<OverlordUsage> => apiRequest<OverlordUsage>('/api/overlord/usage');

export { readUsage };
