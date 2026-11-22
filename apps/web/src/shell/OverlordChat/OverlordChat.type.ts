/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { ActionState, ApprovalDecision, OverlordApproval, OverlordRule, RuleDraft } from '../../data/overlord';

type TalkKind = 'assistant' | 'owner' | 'wake';

interface TalkRow {
  kind: TalkKind;
  key: string;
  text: string;
}

interface ToolsRow {
  kind: 'tools';
  key: string;
  names: string[];
}

type TranscriptRow = TalkRow | ToolsRow;

interface ContentBlock {
  type?: string;
  text?: string;
  name?: string;
}

type Piece = Omit<TalkRow, 'key'> | { kind: 'tool'; name: string };

interface ChatBoxProps {
  running: boolean;
  action: ActionState;
  onSend: (text: string) => Promise<boolean>;
}

type ChatTab = 'chat' | 'rules' | 'queue';

type QueueSectionName = 'wakes' | 'held' | 'approvals' | 'alerts';

interface RuleRowProps {
  rule: OverlordRule;
  busy: boolean;
  onToggle: (enabled: boolean) => void;
  onEdit: () => void;
  onRun: () => void;
  onDelete: () => void;
}

interface RuleFormProps {
  rule: OverlordRule | null;
  onClose: () => void;
}

interface RuleFormState {
  draft: RuleDraft;
  action: ActionState;
  change: (patch: Partial<RuleDraft>) => void;
  save: () => void;
}

interface QueueSectionProps {
  title: string;
  count: number;
  children: ReactNode;
}

interface QueueLineProps {
  text: string | null;
  label?: string | null;
  at?: string;
}

interface ApprovalLineProps {
  approval: OverlordApproval;
  busy: boolean;
  onDecide: (id: string, decision: ApprovalDecision) => void;
}

interface ChatOpen {
  open: boolean;
  toggle: () => void;
  close: () => void;
}

export type {
  ApprovalLineProps,
  ChatBoxProps,
  ChatTab,
  QueueLineProps,
  QueueSectionName,
  QueueSectionProps,
  RuleFormProps,
  RuleFormState,
  RuleRowProps,
  ChatOpen,
  ContentBlock,
  Piece,
  TranscriptRow,
};
