/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { OverlordRule, RulePatch } from './overlord.type';

const updateRule = (id: string, patch: RulePatch): Promise<OverlordRule> =>
  apiRequest<OverlordRule>(`/api/overlord/rules/${encodeURIComponent(id)}`, { method: 'PUT', json: patch });

export { updateRule };
