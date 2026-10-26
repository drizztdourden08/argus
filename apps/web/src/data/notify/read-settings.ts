/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { NotifySettings } from '../types';

const readSettings = (): Promise<NotifySettings> => apiRequest<NotifySettings>('/api/notify/settings');

export { readSettings };
