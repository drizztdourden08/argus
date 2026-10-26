/* @layer renderer-app @kind logic */
import { apiRequest } from './api-request';
import { scopePath } from './scope-path';
import type { BatchOp, BatchResult } from './api.type';

const batchDocs = (scope: string, ops: BatchOp[]): Promise<BatchResult> =>
  apiRequest<BatchResult>(scopePath(scope, 'db', 'batch'), { method: 'POST', json: { ops } });

export { batchDocs };
