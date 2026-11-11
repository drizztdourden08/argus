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
  cli?: { loggedIn?: boolean; expiresAt?: number | null; orgId?: string | null };
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

type CompareStatus =
  | 'same' | 'missing' | 'will_create' | 'only_here' | 'folder_differs'
  | 'archived_here_only' | 'active_here_only' | 'not_subthread' | 'subthread';

interface CompareCell {
  present?: boolean;
  folder?: string | null;
  archived?: boolean;
  parentId?: string | null;
  status?: CompareStatus;
}

interface CompareRow {
  id: string;
  kind: 'folder' | 'chat';
  depth: number;
  parentId?: string | null;
  title: string;
  cells: Record<string, CompareCell | undefined>;
}

interface CompareAccount {
  key: string;
  label?: string;
  color?: string;
  scope?: string;
}

interface CompareDoc {
  scannedAt?: string | null;
  accounts: CompareAccount[];
  rows: CompareRow[];
  counts?: { same?: number; different?: number; byKind?: Record<string, number> };
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
  | 'create_folder' | 'assign_folder' | 'write_storage' | 'verify';

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

export type {
  AccountDoc, AccountLimit, AccountRules, AccountSeverity, AccountsDrift, AccountsRun, CompareAccount, CompareCell, CompareDoc,
  CompareRow, CompareStatus, RunPhase, SyncStepDoc, SyncStepKind,
};
