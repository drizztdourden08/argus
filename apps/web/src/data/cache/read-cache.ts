/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { CacheReport } from './cache.type';

const readCache = (): Promise<CacheReport> => apiRequest<CacheReport>('/api/cache');

export { readCache };
