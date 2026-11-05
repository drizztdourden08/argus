/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { AutoAnswer, AutoPatch } from './notify.type';

const writeAuto = (patch: AutoPatch): Promise<AutoAnswer> =>
  apiRequest<AutoAnswer>('/api/notify/auto', { method: 'POST', json: patch });

export { writeAuto };
