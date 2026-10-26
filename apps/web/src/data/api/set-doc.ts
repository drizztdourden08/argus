/* @layer renderer-app @kind logic */
import { apiRequest } from './api-request';
import { scopePath } from './scope-path';
import type { DocData, PostResult } from './api.type';

const setDoc = (scope: string, coll: string, id: string, data: DocData): Promise<PostResult> =>
  apiRequest<PostResult>(scopePath(scope, 'db', coll, id), { method: 'PUT', json: { data } });

export { setDoc };
