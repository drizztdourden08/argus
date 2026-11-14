/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { ModeSwitch, OverlordMode } from './overlord.type';

const switchMode = (mode: OverlordMode): Promise<ModeSwitch> =>
  apiRequest<ModeSwitch>('/api/overlord/mode', { method: 'POST', json: { mode } });

export { switchMode };
