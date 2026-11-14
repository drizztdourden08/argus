/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useSearchParams } from 'react-router';
import {
  APPROVALS_POLL_MS, SPEND_POLL_MS, STATUS_POLL_MS,
  controlOverlord, decideApproval, readApprovals, readSpend, readStatus, saveBudget, sendMessage, useAction, useOverlordTranscript,
} from '../../../../data/overlord';
import type { ApprovalDecision, OverlordAction } from '../../../../data/overlord';
import { useRemoteRead } from '../../../../data/remote';
import { APPROVAL_PARAM } from '../overlord.constants';
import { pendingApprovals, toRows } from '../model';

const noop = (): void => {};

const useOverlord = () => {
  const [params] = useSearchParams();
  const status = useRemoteRead(readStatus, STATUS_POLL_MS);
  const spend = useRemoteRead(readSpend, SPEND_POLL_MS);
  const approvals = useRemoteRead(readApprovals, APPROVALS_POLL_MS);
  const feed = useOverlordTranscript(status.data?.sessionId ?? null);
  const control = useAction(status.reload);
  const decision = useAction(() => {
    approvals.reload();
    status.reload();
  });
  const budget = useAction(spend.reload);
  const chat = useAction(noop);
  const rows = useMemo(() => toRows(feed.lines), [feed.lines]);
  const pending = useMemo(() => pendingApprovals(approvals.data ?? [], params.get(APPROVAL_PARAM)), [approvals.data, params]);

  return {
    status: status.data,
    spend: spend.data,
    problem: status.error,
    pending,
    rows,
    transcriptProblem: feed.problem,
    control,
    decision,
    budget,
    chat,
    onControl: (action: OverlordAction) => void control.run(() => controlOverlord(action)),
    onDecide: (id: string, choice: ApprovalDecision) => void decision.run(() => decideApproval(id, choice)),
    onSave: (budgetUsd: number) => void budget.run(() => saveBudget(budgetUsd)),
    onSend: (text: string) => chat.run(() => sendMessage(text)),
  };
};

export { useOverlord };
