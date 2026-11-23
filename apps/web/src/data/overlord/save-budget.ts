/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { OverlordSpend } from './overlord.type';

const saveBudget = (budgetUsd: number | null): Promise<OverlordSpend> =>
  apiRequest<OverlordSpend>('/api/overlord/budget', { method: 'PUT', json: { budgetUsd } });

export { saveBudget };
