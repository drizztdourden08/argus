/* @layer renderer-app @kind types */
import type { ActionState, OverlordAction, OverlordApproval, OverlordSpend, OverlordStatus } from '../../../data/overlord';

interface StatusBarProps {
  status: OverlordStatus;
  action: ActionState;
  onControl: (action: OverlordAction) => void;
  chatOpen: boolean;
  onChat: () => void;
}

interface ApprovalStripProps {
  approvals: OverlordApproval[];
  action: ActionState;
  onDecide: (id: string, decision: 'allow' | 'deny') => void;
}

interface SpendCardProps {
  spend: OverlordSpend;
}

interface BudgetCardProps {
  spend: OverlordSpend;
  action: ActionState;
  onSave: (budgetUsd: number) => void;
}

export type { ApprovalStripProps, BudgetCardProps, SpendCardProps, StatusBarProps };
