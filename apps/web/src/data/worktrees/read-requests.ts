/* @layer renderer-app @kind logic */
import { apiRequest, scopePath } from '../api';
import { REQUESTS_PATH } from './worktrees.constants';
import type { RequestRecord } from './worktrees.type';

const readRequests = async (key: string): Promise<RequestRecord[]> =>
  (await apiRequest<{ requests: RequestRecord[] }>(scopePath(key, ...REQUESTS_PATH))).requests;

export { readRequests };
