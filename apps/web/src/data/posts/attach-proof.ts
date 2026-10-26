/* @layer renderer-app @kind logic */
import { apiRequest, queryString, scopePath } from '../api';
import type { ProofResult } from './posts.type';

const attachProof = (key: string, testId: string, image: Blob, caption?: string): Promise<ProofResult> =>
  apiRequest<ProofResult>(`${scopePath(key, 'tests', testId, 'proofs')}${queryString({ caption })}`, { method: 'POST', raw: image });

export { attachProof };
