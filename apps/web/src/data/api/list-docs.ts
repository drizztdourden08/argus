/* @layer renderer-app @kind logic */
import { apiRequest } from './api-request';
import { scopePath } from './scope-path';
import type { ListResult } from './api.type';

const listDocs = (scope: string, coll: string): Promise<ListResult> => apiRequest<ListResult>(scopePath(scope, 'db', coll));

export { listDocs };
