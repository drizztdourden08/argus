/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { ActionState, ApprovalDecision, OverlordApproval, OverlordQueue, OverlordRule, RuleDraft } from '../../data/overlord';

type Editing = OverlordRule | 'new' | null;

type TalkKind = 'assistant' | 'owner' | 'wake' | 'rule' | 'thinking' | 'turn' | 'approval';

interface TalkRow {
  kind: TalkKind;
  key: string;
  text: string;
  id?: string;
}

interface ToolItem {
  id: string;
  name: string;
  summary: string;
  status: 'running' | 'done' | 'failed';
}

interface ToolsRow {
  kind: 'tools';
  key: string;
  tools: ToolItem[];
}

interface StepContentProps {
  row: TalkRow;
}

type TranscriptRow = TalkRow | ToolsRow;

interface ContentBlock {
  type?: string;
  text?: string;
  name?: string;
  id?: string;
  input?: unknown;
  thinking?: string;
  tool_use_id?: string;
  is_error?: boolean;
  content?: unknown;
}

type Piece =
  | Omit<TalkRow, 'key'>
  | { kind: 'tool'; id: string; name: string; summary: string }
  | { kind: 'result'; id: string; failed: boolean; text: string };

interface QueueFeed {
  queue?: OverlordQueue | null;
  problem?: string | null;
  action: ActionState;
  reload: () => void;
  decide: (id: string, decision: ApprovalDecision) => Promise<boolean>;
}

interface QueueFeedProps {
  feed: QueueFeed;
}

interface PendingSend {
  id: number;
  text: string;
  baseline: number;
  state: 'sending' | 'sent';
}

interface ChatBoxProps {
  running: boolean;
  note: string;
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
  PendingSend,
  StepContentProps,
  TalkKind,
  TalkRow,
  ToolsRow,
  QueueFeedProps,
  ToolItem,
  ChatTab,
  Editing,
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
