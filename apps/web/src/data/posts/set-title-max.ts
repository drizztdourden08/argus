/* @layer renderer-app @kind logic */
import { apiRequest, scopePath } from '../api';
import type { PostResult } from '../api';
import { ownerEntry } from './owner-entry';

const setTitleMax = (key: string, titleMax: number | null): Promise<PostResult> =>
  apiRequest<PostResult>(scopePath(key, 'entries'), { method: 'POST', json: ownerEntry({ kind: 'title-max', titleMax }) });

export { setTitleMax };
