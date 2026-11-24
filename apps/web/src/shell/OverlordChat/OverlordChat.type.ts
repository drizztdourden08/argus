/* @layer renderer-app @kind types */
import type { ChatTranscriptMessage, ChatTranscriptRole } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { ActionState, OverlordAction, OverlordApproval, OverlordMode, OverlordSpend, OverlordStatus } from '../../data/overlord';

type TalkKind = 'assistant' | 'owner' | 'wake' | 'rule' | 'thinking' | 'turn' | 'approval';

interface TalkRow {
  kind: TalkKind;
  key: string;
  text: string;
  id?: string;
  at?: string;
  live?: boolean;
}

type ToolStatus = 'running' | 'done' | 'failed';

interface ToolItem {
  id: string;
  name: string;
  summary: string;
  status: ToolStatus;
  error?: string;
}

interface ToolsRow {
  kind: 'tools';
  key: string;
  tools: ToolItem[];
  at?: string;
}

interface WorkingActivity {
  text: string;
  mark: 'thinking' | 'working';
}

interface DraftBlock {
  kind: string;
  name: string | null;
  text: string;
}

interface LiveDraft {
  rows: TalkRow[];
  tool: string | null;
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

interface ChatFeedProps {
  messages: ChatTranscriptMessage[];
  queued: ChatTranscriptMessage[];
  working: WorkingActivity | null;
}

interface WorkingLineProps {
  activity: WorkingActivity;
}

interface MessageBodyProps {
  role: ChatTranscriptRole;
  content: ReactNode;
}

interface ToolsGroupProps {
  tools: ToolItem[];
}

interface ToolLineProps {
  tool: ToolItem;
}

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

export type { ApprovalStripProps, ModeSectionProps, SpendSectionProps, StatusSectionProps, ChatBoxProps, ChatFeedProps,ContentBlock, DraftBlock, LiveDraft, MessageBodyProps, PendingSend, Piece, StepContentProps, TalkKind, TalkRow, ToolItem, ToolLineProps, ToolStatus, ToolsGroupProps, ToolsRow, TranscriptRow, WorkingActivity, WorkingLineProps };
