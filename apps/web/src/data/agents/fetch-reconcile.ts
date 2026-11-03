/* @layer renderer-app @kind logic */
import { apiRequest, queryString } from '../api';
import type { ReconcileReport } from './agents.type';

const fetchReconcile = async (key: string): Promise<ReconcileReport> => {
  const body = await apiRequest<Partial<ReconcileReport>>(`/api/agents/reconcile${queryString({ project: key })}`);
  return { ran: body.ran ?? false, at: body.at ?? null, findings: Array.isArray(body.findings) ? body.findings : [] };
};

export { fetchReconcile };
