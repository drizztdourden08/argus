/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { OverlordApproval } from './overlord.type';

const readApprovals = async (): Promise<OverlordApproval[]> => {
  const body = await apiRequest<{ approvals: OverlordApproval[] }>('/api/overlord/approvals');
  return body.approvals;
};

export { readApprovals };
