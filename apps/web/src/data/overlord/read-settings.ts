/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { OverlordSettings } from './overlord.type';

const readSettings = (): Promise<OverlordSettings> => apiRequest<OverlordSettings>('/api/overlord/settings');

export { readSettings };
