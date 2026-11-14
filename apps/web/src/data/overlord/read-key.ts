/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { KeyStatus } from './overlord.type';

const readKey = (): Promise<KeyStatus> => apiRequest<KeyStatus>('/api/overlord/key');

export { readKey };
