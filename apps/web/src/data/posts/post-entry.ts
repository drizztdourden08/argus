/* @layer renderer-app @kind logic */
import { apiRequest, scopePath } from '../api';
import type { DocData, PostResult } from '../api';

const postEntry = (key: string, entry: DocData): Promise<PostResult> =>
  apiRequest<PostResult>(scopePath(key, 'entries'), { method: 'POST', json: entry });

export { postEntry };
