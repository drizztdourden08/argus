/* @layer renderer-app @kind types */
import type { ActionState, OverlordAction, OverlordApproval, OverlordSpend, OverlordStatus } from '../../../data/overlord';
import type { TalkRow, TranscriptRow } from './model';

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

interface TranscriptViewProps {
  rows: TranscriptRow[];
  problem: string | null;
}

interface ChatBoxProps {
  running: boolean;
  action: ActionState;
  onSend: (text: string) => Promise<boolean>;
}

interface ChatDrawerProps {
  open: boolean;
  rows: TranscriptRow[];
  problem: string | null;
  running: boolean;
  action: ActionState;
  onSend: (text: string) => Promise<boolean>;
  onClose: () => void;
}

interface SpendCardProps {
  spend: OverlordSpend;
}

interface BudgetCardProps {
  spend: OverlordSpend;
  action: ActionState;
  onSave: (budgetUsd: number) => void;
}

interface TalkRowViewProps {
  row: TalkRow;
}

interface ToolsRowProps {
  names: string[];
}

export type { ApprovalStripProps, BudgetCardProps, ChatBoxProps, ChatDrawerProps, SpendCardProps, StatusBarProps, TalkRowViewProps, ToolsRowProps, TranscriptViewProps };
