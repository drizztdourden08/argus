/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { KeyStatus } from './overlord.type';

const removeKey = (): Promise<KeyStatus> => apiRequest<KeyStatus>('/api/overlord/key', { method: 'DELETE' });

export { removeKey };
