/* @layer renderer-app @kind logic */
import { apiRequest, scopePath } from '../api';
import type { PostResult } from '../api';
import { ownerEntry } from './owner-entry';

const setAvatar = (key: string, avatar: string | null): Promise<PostResult> =>
  apiRequest<PostResult>(scopePath(key, 'entries'), { method: 'POST', json: ownerEntry({ kind: 'avatar', avatar }) });

export { setAvatar };
