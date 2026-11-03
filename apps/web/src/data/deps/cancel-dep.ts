/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { DepAnswer } from './deps.type';

const cancelDep = (id: string): Promise<DepAnswer> =>
  apiRequest<DepAnswer>(`/api/deps/${encodeURIComponent(id)}/cancel`, { method: 'POST', json: { actor: 'owner' } });

export { cancelDep };
