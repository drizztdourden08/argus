/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { PruneResult } from './cache.type';

const pruneCache = (): Promise<PruneResult> => apiRequest<PruneResult>('/api/cache/prune', { method: 'POST', json: {} });

export { pruneCache };
