/* @layer renderer-app @kind logic */
import { apiRequest, scopePath } from '../api';
import type { PostResult } from '../api';
import type { TestOutcome } from './posts.type';

const recordResult = (key: string, testId: string, outcome: TestOutcome, text?: string): Promise<PostResult> =>
  apiRequest<PostResult>(scopePath(key, 'tests', testId, 'result'), { method: 'POST', json: { outcome, text } });

export { recordResult };
