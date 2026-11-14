/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { OverlordAction, OverlordStatus } from './overlord.type';

const controlOverlord = (action: OverlordAction): Promise<OverlordStatus> =>
  apiRequest<OverlordStatus>('/api/overlord/control', { method: 'POST', json: { action } });

export { controlOverlord };
