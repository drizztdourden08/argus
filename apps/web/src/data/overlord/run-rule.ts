/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';

const runRule = (id: string): Promise<unknown> =>
  apiRequest(`/api/overlord/rules/${encodeURIComponent(id)}/run`, { method: 'POST' });

export { runRule };
