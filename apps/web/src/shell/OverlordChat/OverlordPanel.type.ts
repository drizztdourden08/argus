/* @layer renderer-app @kind types */
import type { ActionState, OverlordAction, OverlordApproval, OverlordMode, OverlordSpend, OverlordStatus } from '../../data/overlord';

interface StatusSectionProps {
  status: OverlordStatus;
  action: ActionState;
  onControl: (action: OverlordAction) => void;
}

interface ApprovalStripProps {
  approvals: OverlordApproval[];
  action: ActionState;
  onDecide: (id: string, decision: 'allow' | 'deny') => void;
}

interface SpendSectionProps {
  spend: OverlordSpend;
}

interface ModeSectionProps {
  mode: string | null;
  action: ActionState;
  onSwitch: (mode: OverlordMode) => void;
}

export type { ApprovalStripProps, ModeSectionProps, SpendSectionProps, StatusSectionProps };
