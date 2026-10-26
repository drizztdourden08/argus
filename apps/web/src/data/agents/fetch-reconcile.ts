/* @layer renderer-app @kind logic */
import { apiRequest, queryString } from '../api';
import type { ReconcileReport } from './agents.type';

const fetchReconcile = (key: string): Promise<ReconcileReport> =>
  apiRequest<ReconcileReport>(`/api/agents/reconcile${queryString({ project: key })}`);

export { fetchReconcile };
