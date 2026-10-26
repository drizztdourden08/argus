/* @layer renderer-app @kind logic */
import { apiRequest } from './api-request';
import { scopePath } from './scope-path';
import type { DocResult } from './api.type';

const getDoc = (scope: string, coll: string, id: string): Promise<DocResult> =>
  apiRequest<DocResult>(scopePath(scope, 'db', coll, id));

export { getDoc };
