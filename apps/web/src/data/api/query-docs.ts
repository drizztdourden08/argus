/* @layer renderer-app @kind logic */
import { apiRequest } from './api-request';
import { scopePath } from './scope-path';
import type { DbQuery, ListResult } from './api.type';

const queryDocs = (scope: string, coll: string, query: DbQuery): Promise<ListResult> =>
  apiRequest<ListResult>(scopePath(scope, 'db', coll, 'query'), { method: 'POST', json: query });

export { queryDocs };
