/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';

const removeRule = (id: string): Promise<unknown> =>
  apiRequest(`/api/overlord/rules/${encodeURIComponent(id)}`, { method: 'DELETE' });

export { removeRule };
