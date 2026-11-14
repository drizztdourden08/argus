/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { KeyStatus } from './overlord.type';

const saveKey = (key: string): Promise<KeyStatus> => apiRequest<KeyStatus>('/api/overlord/key', { method: 'PUT', json: { key } });

export { saveKey };
