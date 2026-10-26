/* @layer renderer-app @kind logic */
import { apiRequest, scopePath } from '../api';
import type { PostResult } from '../api';
import { ownerEntry } from './owner-entry';

const rerunTest = (key: string, testId: string): Promise<PostResult> =>
  apiRequest<PostResult>(scopePath(key, 'entries'), { method: 'POST', json: ownerEntry({ kind: 'rerun', replyTo: testId }) });

export { rerunTest };
