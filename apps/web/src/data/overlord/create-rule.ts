/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { OverlordRule, RuleDraft } from './overlord.type';

const createRule = (draft: RuleDraft): Promise<OverlordRule> =>
  apiRequest<OverlordRule>('/api/overlord/rules', { method: 'POST', json: draft });

export { createRule };
