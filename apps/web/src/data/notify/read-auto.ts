/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { AutoAnswer } from './notify.type';

const readAuto = (): Promise<AutoAnswer> => apiRequest<AutoAnswer>('/api/notify/auto');

export { readAuto };
