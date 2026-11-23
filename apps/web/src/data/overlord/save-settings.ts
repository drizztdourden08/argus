/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { OverlordSettings, OverlordSettingsPatch } from './overlord-settings.type';

const saveSettings = (patch: OverlordSettingsPatch): Promise<OverlordSettings> =>
  apiRequest<OverlordSettings>('/api/overlord/settings', { method: 'POST', json: patch });

export { saveSettings };
