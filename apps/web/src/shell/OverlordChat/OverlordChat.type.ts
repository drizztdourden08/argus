/* @layer renderer-app @kind types */
import type { ChatTranscriptMessage, ChatTranscriptRole } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { ActionState, OverlordQueue, OverlordRule, OverlordSettings, OverlordUsage, RuleDraft } from '../../data/overlord';

type Editing = OverlordRule | 'new' | null;

type TalkKind = 'assistant' | 'owner' | 'wake' | 'rule' | 'thinking' | 'turn' | 'approval';

interface TalkRow {
  kind: TalkKind;
  key: string;
  text: string;
  id?: string;
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

type QueueSectionName = 'wakes' | 'held' | 'alerts';

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

interface ChatFeedProps {
  messages: ChatTranscriptMessage[];
  queued: ChatTranscriptMessage[];
  working: boolean;
}

interface ChatMessageProps {
  message: ChatTranscriptMessage;
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

interface SettingChoice {
  id: string;
  label: string;
}

type SettingKey = 'model' | 'effort' | 'permissionMode';

interface SettingChipProps {
  label: string;
  current: string;
  selected: string;
  choices: SettingChoice[];
  busy: boolean;
  onPick: (id: string) => void;
}

interface ChatSettings {
  settings: OverlordSettings | null;
  busy: boolean;
  problem: string | null;
  pick: (key: SettingKey, id: string) => void;
}

interface ChatUsage {
  usage: OverlordUsage | null;
  reload: () => void;
}

interface ChatBarProps {
  working: boolean;
}

interface UsageRingProps {
  usage: ChatUsage;
}

interface UsagePanelProps {
  usage: OverlordUsage;
}

interface UsageRow {
  label: string;
  value: string;
}

interface UsageSection {
  title: string;
  rows: UsageRow[];
}

interface ChipSpec {
  key: SettingKey;
  label: string;
  current: string;
  selected: string;
  choices: SettingChoice[];
}

interface SettingChipsProps {
  chat: ChatSettings;
}

interface ChatOpen {
  open: boolean;
  toggle: () => void;
  close: () => void;
}

export type {
  ChatBarProps,
  MessageBodyProps,
  ChipSpec,
  SettingChipsProps,
  UsageSection,
  ChatSettings,
  ChatUsage,
  SettingChipProps,
  SettingChoice,
  SettingKey,
  ToolLineProps,
  ToolStatus,
  ToolsGroupProps,
  UsagePanelProps,
  UsageRingProps,
  UsageRow,
  ChatFeedProps,
  ChatMessageProps,
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
