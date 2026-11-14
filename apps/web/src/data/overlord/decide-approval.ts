/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { ApprovalDecision, OverlordApproval } from './overlord.type';

const decideApproval = (id: string, decision: ApprovalDecision): Promise<OverlordApproval> =>
  apiRequest<OverlordApproval>(`/api/overlord/approvals/${encodeURIComponent(id)}`, { method: 'POST', json: { decision } });

export { decideApproval };
