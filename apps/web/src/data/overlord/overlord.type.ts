/* @layer renderer-app @kind types */
interface KeyStatus {
  set: boolean;
  last4: string | null;
  savedAt: string | null;
}

interface OverlordKeyApi {
  status: KeyStatus | null;
  busy: boolean;
  problem: string | null;
  save: (key: string) => Promise<boolean>;
  remove: () => Promise<boolean>;
}

type OverlordState = 'stopped' | 'starting' | 'idle' | 'turn' | 'paused-budget' | 'waiting-approval' | 'crashed';

type OverlordMode = 'off' | 'shadow' | 'live';

interface ModeSwitch {
  from: OverlordMode;
  to: OverlordMode;
  already: boolean;
}

type OverlordAction = 'restart';

type OverlordActivity = 'off' | 'idle' | 'thinking' | 'working';

interface OverlordStatus {
  mode: string | null;
  status: OverlordState;
  pid: number | null;
  startedAt: string | null;
  sessionId: string | null;
  activity?: OverlordActivity;
  problem?: string | null;
  nextRuleAt?: string | number | null;
  nextRuleEveryMin?: number | null;
  nextRuleTitle?: string | null;
}

type RuleAuthor = 'owner' | 'overlord';

interface RuleDraft {
  title: string;
  prompt: string;
  everyMin: number;
}

interface OverlordRule extends RuleDraft {
  id: string;
  enabled: boolean;
  createdBy: RuleAuthor;
  lastRunAt?: string | null;
  nextRunAt?: string | null;
}

interface RulesPage {
  rules: OverlordRule[];
  nextDueAt?: string | null;
}

type RulePatch = Partial<RuleDraft & { enabled: boolean }>;

interface QueueWake {
  reason: string | null;
  text: string | null;
  at: string;
}

type QueueHeld = string | { text: string; at?: string };

interface QueueAlert {
  text: string;
  at?: string;
}

interface OverlordQueue {
  wakes: QueueWake[];
  held: QueueHeld[];
  approvals: OverlordApproval[];
  alerts: QueueAlert[];
}

interface OverlordSpend {
  month: string;
  spentUsd: number;
  budgetUsd: number | null;
  warnAt: number;
  leftUsd: number | null;
  note: string;
}

interface OverlordChoice {
  id: string;
  label: string;
}

type LimitSource = 'set' | 'board' | 'default';

interface LimitEntry<T> {
  value: T;
  source: LimitSource;
}

interface OverlordSettings {
  model: string;
  effort: string;
  permissionMode: string;
  choices: { models: OverlordChoice[]; efforts: string[]; permissionModes: OverlordChoice[] };
  cwd: LimitEntry<string>;
  claudeExe: LimitEntry<string>;
  budgetUsd: LimitEntry<number | null>;
  restartMax: LimitEntry<number>;
  approvalTimeoutMin: LimitEntry<number>;
}

type LimitKey = 'cwd' | 'claudeExe' | 'budgetUsd' | 'restartMax' | 'approvalTimeoutMin';

/** An empty text or an empty string in place of a number resets the field to its default. */
interface OverlordSettingsPatch {
  model?: string;
  effort?: string;
  permissionMode?: string;
  cwd?: string;
  claudeExe?: string;
  budgetUsd?: number | '';
  restartMax?: number | '';
  approvalTimeoutMin?: number | '';
}

/** The settings form; a number that is not finite is a blank field. */
interface SettingsDraft {
  cwd: string;
  claudeExe: string;
  budgetUsd: number;
  restartMax: number;
  approvalTimeoutMin: number;
}

interface UsageTotals {
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  costUsd: number;
  turns: number;
  durationMs: number;
}

interface OverlordUsage {
  session: UsageTotals;
  lastTurn: UsageTotals | null;
  context: { usedTokens: number; windowTokens: number };
  model: string;
  month: { spentUsd: number; budgetUsd: number | null };
  updatedAt: string;
}

type ApprovalState = 'pending' | 'allowed' | 'denied' | 'expired';

type ApprovalDecision = 'allow' | 'deny';

interface OverlordApproval {
  id: string;
  at: string;
  tool: string;
  inputSummary: string;
  status: ApprovalState;
}

interface TranscriptEvent {
  type?: string;
  message?: { content?: unknown };
  duration_ms?: number;
  argus?: { source?: string };
  event?: string;
  id?: string;
  tool?: string;
  summary?: string;
  status?: string;
  decidedBy?: string;
}

interface TranscriptLine {
  index: number;
  event: TranscriptEvent | string;
}

interface TranscriptPage {
  session: string | null;
  lines: (TranscriptEvent | string)[];
  next: number;
}

interface ActionState {
  busy: boolean;
  problem: string | null;
  run: (action: () => Promise<unknown>) => Promise<boolean>;
}

interface TranscriptFeed {
  lines: TranscriptLine[];
  problem: string | null;
}

export type {
  ActionState,
  OverlordActivity,
  OverlordQueue,
  OverlordRule,
  QueueHeld,
  RuleDraft,
  RulePatch,
  RulesPage,
  ApprovalDecision,
  KeyStatus,
  ModeSwitch,
  OverlordAction,
  OverlordApproval,
  OverlordKeyApi,
  OverlordMode,
  LimitEntry,
  LimitKey,
  LimitSource,
  OverlordSettings,
  OverlordSettingsPatch,
  SettingsDraft,
  OverlordSpend,
  OverlordUsage,
  UsageTotals,
  OverlordState,
  OverlordStatus,
  TranscriptEvent,
  TranscriptFeed,
  TranscriptLine,
  TranscriptPage,
};
