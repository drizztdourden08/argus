/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useSearchParams } from 'react-router';
import {
  APPROVALS_POLL_MS, SPEND_POLL_MS, STATUS_POLL_MS,
  controlOverlord, decideApproval, readApprovals, readSpend, readStatus, saveBudget, switchMode, useAction,
} from '../../../../data/overlord';
import type { ApprovalDecision, OverlordAction, OverlordMode } from '../../../../data/overlord';
import { useRemoteRead } from '../../../../data/remote';
import { APPROVAL_PARAM } from '../overlord.constants';
import { pendingApprovals } from '../model';

const useOverlord = () => {
  const [params] = useSearchParams();
  const status = useRemoteRead(readStatus, STATUS_POLL_MS);
  const spend = useRemoteRead(readSpend, SPEND_POLL_MS);
  const approvals = useRemoteRead(readApprovals, APPROVALS_POLL_MS);
  const control = useAction(status.reload);
  const decision = useAction(() => {
    approvals.reload();
    status.reload();
  });
  const budget = useAction(spend.reload);
  const modeAction = useAction(status.reload);
  const pending = useMemo(() => pendingApprovals(approvals.data ?? [], params.get(APPROVAL_PARAM)), [approvals.data, params]);

  return {
    status: status.data,
    spend: spend.data,
    problem: status.error,
    pending,
    control,
    decision,
    budget,
    modeAction,
    onControl: (action: OverlordAction) => void control.run(() => controlOverlord(action)),
    onDecide: (id: string, choice: ApprovalDecision) => void decision.run(() => decideApproval(id, choice)),
    onSwitch: (mode: OverlordMode) => void modeAction.run(() => switchMode(mode)),
    onSave: (budgetUsd: number | null) => void budget.run(() => saveBudget(budgetUsd)),
  };
};

export { useOverlord };
