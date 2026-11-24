/* @layer renderer-app @kind types */
type AccountSeverity = 'normal' | 'warning' | 'critical';

interface AccountLimit {
  kind?: string;
  group?: string;
  percent?: number;
  severity?: string;
  resetsAt?: string | null;
  model?: string | null;
  isActive?: boolean;
}

interface AccountLimitState {
  confirmed?: boolean;
  atMs?: number | null;
  untilMs?: number | null;
}

interface AccountRules {
  autoSwitch?: boolean;
  switchAt?: number;
  handover?: 'off' | 'auto' | number;
  canReceive?: boolean;
}

interface AccountDoc {
  key?: string;
  label?: string;
  color?: string;
  active?: boolean;
  plan?: string | null;
  email?: string | null;
  accountId?: string | null;
  cli?: { loggedIn?: boolean; loginExpiresAt?: string | null; orgId?: string | null };
  desktopLogin?: { saved?: boolean; savedAt?: string | null };
  limits?: AccountLimit[];
  limit?: AccountLimitState;
  readAt?: string | null;
  error?: string | null;
  burnPerMin?: number | null;
  rules?: AccountRules;
  chats?: { total?: number; archived?: number; folders?: number };
}

type NextWhen = { kind: 'percent'; percent: number } | { kind: 'time'; atMs: number } | { kind: 'unknown' };

interface NextSwitch {
  from?: string | null;
  to?: string | null;
  source?: 'owner' | 'auto';
  when?: NextWhen;
}

type NextRunPhase = 'idle' | 'handover-asked' | 'handover-done' | 'limit-confirmed' | 'switching' | 'switched' | 'failed';

interface NextRun {
  phase?: NextRunPhase;
  atMs?: number | null;
}

interface AccountsStatus {
  next?: NextSwitch;
  run?: NextRun;
}

interface AccountsDrift {
  scannedAt?: string | null;
  total?: number;
  byKind?: Record<string, number>;
}

type RunPhase = 'idle' | 'scan' | 'waiting_for_quit' | 'plan' | 'backup' | 'apply' | 'verify' | 'done' | 'failed';

interface AccountsRun {
  phase?: RunPhase;
  detail?: string | null;
  step?: number;
  total?: number;
  backupDir?: string | null;
  startedAt?: string | null;
  finishedAt?: string | null;
  result?: RunResult | string | null;
}

type RunResult = {
  error?: string;
  copies?: number;
  updates?: number;
  writes?: number;
  verified?: boolean;
};

type SyncStepKind =
  | 'backup' | 'copy_session' | 'update_session' | 'link_thread' | 'archive'
  | 'create_folder' | 'assign_folder' | 'write_storage' | 'verify'
  | 'move_chat' | 'delete_chat' | 'rename_folder' | 'delete_folder' | 'set_view';

interface SyncStepDoc {
  seq?: number;
  kind?: SyncStepKind;
  account?: string;
  sessionId?: string | null;
  title?: string | null;
  folder?: string | null;
  status?: 'running' | 'done' | 'failed';
  at?: string | null;
  detail?: string | null;
}

type DesktopPhase =
  | 'idle' | 'countdown' | 'handover' | 'closing' | 'waiting_for_quit' | 'saving_login' | 'syncing' | 'restoring_login'
  | 'scheduling_resume' | 'starting' | 'confirming' | 'verifying' | 'done' | 'cancelled' | 'failed';

type SwitchPhase = 'idle' | 'countdown' | 'handover' | 'sync' | 'switching' | 'verifying' | 'done' | 'failed' | 'cancelled';

type SwitchStage = 'detect' | 'countdown' | 'handover' | 'sync' | 'switch' | 'verify';

type SwitchStepState = 'waiting' | 'running' | 'done' | 'failed' | 'skipped';

interface SwitchStepDoc {
  at?: string | null;
  stage?: SwitchStage;
  state?: SwitchStepState;
  text?: string | null;
  detail?: string | null;
  id?: string;
  label?: string | null;
  status?: string;
}

type SwitchThreadState = 'found' | 'asked' | 'ready' | 'skipped' | 'failed';

interface SwitchThreadDoc {
  board?: string;
  thread?: string;
  title?: string | null;
  sessionId?: string | null;
  state?: SwitchThreadState;
  note?: string | null;
}

type SwitchSyncState = 'pending' | 'running' | 'done' | 'failed' | 'skipped';

interface SwitchSyncCounts {
  copied?: number;
  updated?: number;
  folders?: number;
  archived?: number;
}

interface SwitchSyncDoc {
  state?: SwitchSyncState;
  backupDir?: string | null;
  counts?: SwitchSyncCounts | null;
  error?: string | null;
}

interface AccountsSwitch {
  id?: string;
  from?: string | null;
  to?: string | null;
  startedBy?: 'rule' | 'owner';
  phase?: SwitchPhase;
  desktopPhase?: DesktopPhase;
  countdownEndsAt?: number | string | null;
  steps?: SwitchStepDoc[];
  threads?: SwitchThreadDoc[];
  sync?: SwitchSyncDoc | null;
  error?: string | null;
  blocker?: string | null;
}

type HandoverRule = 'off' | 'auto' | number;

interface SettingsRules {
  autoSwitch: boolean;
  switchAt: number;
  handover: HandoverRule;
}

interface SettingsAccount {
  key: string;
  label: string;
  color: string;
  cliDir: string;
  desktopAccount?: string | null;
  desktopOrg?: string | null;
  canReceive: boolean;
  rules: SettingsRules;
}

interface EstimateDefaults {
  opus: number;
  sonnet: number;
  haiku: number;
  lead: number;
  longContextFactor: number;
  safety: number;
}

interface AccountsSettings {
  pollMinutes: number;
  fastPollSeconds: number;
  fastAt: number;
  forceQuit: boolean;
  warnAt: number;
  proposeAt: number;
  archiveRule: 'newest' | 'anywhere';
  estimateDefaults: EstimateDefaults;
  k: number;
  accounts: SettingsAccount[];
}

export type {
  DesktopPhase,
  AccountDoc, AccountLimit, AccountsStatus, AccountSeverity, NextRunPhase, NextWhen, AccountsDrift, AccountsRun, RunPhase, SyncStepDoc, SyncStepKind,
  AccountsSettings, AccountsSwitch, HandoverRule, SettingsAccount, SettingsRules, SwitchPhase, SwitchStage,
  SwitchStepDoc, SwitchStepState, SwitchSyncCounts, SwitchSyncDoc, SwitchThreadDoc, RunResult,
};
