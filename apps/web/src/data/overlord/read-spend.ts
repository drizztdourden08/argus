/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { OverlordSpend } from './overlord.type';

const readSpend = (): Promise<OverlordSpend> => apiRequest<OverlordSpend>('/api/overlord/spend');

export { readSpend };
