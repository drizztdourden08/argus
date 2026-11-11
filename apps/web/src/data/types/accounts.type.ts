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

export type { AccountDoc, AccountLimit, AccountRules, AccountSeverity, AccountsDrift };
