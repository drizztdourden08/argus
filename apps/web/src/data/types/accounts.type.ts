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
  cli?: { loggedIn?: boolean; loginExpiresAt?: string | null; orgId?: string | null };
  desktopLogin?: { saved?: boolean; savedAt?: string | null };
  limits?: AccountLimit[];
  readAt?: string | null;
  error?: string | null;
  burnPerMin?: number | null;
  rules?: AccountRules;
  chats?: { total?: number; archived?: number; folders?: number };
}

interface AccountsDrift {
  scannedAt?: string | null;
  total?: number;
  byKind?: Record<string, number>;
}

type RunPhase = 'idle' | 'scan' | 'plan' | 'backup' | 'apply' | 'verify' | 'done' | 'failed';

interface AccountsRun {
  phase?: RunPhase;
  step?: number;
  total?: number;
  backupDir?: string | null;
  startedAt?: string | null;
  finishedAt?: string | null;
  result?: string | null;
}

type SyncStepKind =
  | 'backup' | 'copy_session' | 'update_session' | 'link_thread' | 'archive'
  | 'create_folder' | 'assign_folder' | 'write_storage' | 'verify'
  | 'move_chat' | 'delete_chat' | 'rename_folder' | 'delete_folder';

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

type SwitchPhase =
  | 'idle' | 'countdown' | 'handover' | 'closing' | 'saving_login' | 'syncing' | 'restoring_login'
  | 'scheduling_resume' | 'starting' | 'confirming' | 'done' | 'cancelled' | 'failed';

type SwitchStepStatus = 'pending' | 'running' | 'done' | 'failed' | 'skipped';

interface SwitchStepDoc {
  id?: string;
  label?: string;
  status?: SwitchStepStatus;
  detail?: string | null;
}

interface AccountsSwitch {
  id?: string;
  from?: string | null;
  to?: string | null;
  startedBy?: 'rule' | 'owner';
  phase?: SwitchPhase;
  countdownEndsAt?: number | string | null;
  steps?: SwitchStepDoc[];
  error?: string | null;
}

interface AccountsCycle {
  id?: string;
  account?: string;
  mode?: 'auto' | 'fixed' | 'now';
  askAt?: number;
  k?: number;
  useAtRequest?: number;
  useAtReady?: number;
  leads?: string[];
}

type HandoverState = 'estimate_asked' | 'estimated' | 'asked' | 'writing' | 'ready' | 'cut_off' | 'resumed' | 'not_resumed';

interface HandoverDoc {
  id?: string;
  cycle?: string;
  board?: string;
  thread?: string;
  sessionId?: string | null;
  state?: HandoverState;
  estimate?: { percent?: number; minutes?: number; agents?: number; at?: number | string | null } | null;
  file?: string | null;
  agents?: { id?: string; branch?: string; stop?: string }[];
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
  warnAt: number;
  proposeAt: number;
  archiveRule: 'newest' | 'anywhere';
  estimateDefaults: EstimateDefaults;
  k: number;
  accounts: SettingsAccount[];
}

export type {
  AccountDoc, AccountLimit, AccountRules, AccountSeverity, AccountsDrift, AccountsRun, RunPhase, SyncStepDoc, SyncStepKind,
  AccountsCycle, AccountsSettings, AccountsSwitch, HandoverDoc, HandoverRule, HandoverState, SettingsAccount,
  SwitchPhase, SwitchStepStatus,
};
