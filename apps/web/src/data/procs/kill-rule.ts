/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { KillRuleResult } from './procs.type';

const killRule = (rule: string): Promise<KillRuleResult> =>
  apiRequest<KillRuleResult>('/api/procs/kill-rule', { method: 'POST', json: { rule, idleOnly: true } });

export { killRule };
