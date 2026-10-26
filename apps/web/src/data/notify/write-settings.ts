/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { NotifySettings } from '../types';
import type { SettingsPatch } from './notify.type';

const writeSettings = (patch: SettingsPatch): Promise<NotifySettings> =>
  apiRequest<NotifySettings>('/api/notify/settings', { method: 'POST', json: patch });

export { writeSettings };
