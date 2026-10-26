/* @layer renderer-app @kind types */
interface KillProcInput {
  pid: number;
  startedAt?: string;
  tree: boolean;
}

interface KillResult {
  killed?: boolean | number | unknown[];
  pid?: number;
  name?: string;
  memMB?: number;
  error?: string;
}

interface KillRuleResult {
  rule?: string;
  killed?: unknown[] | number;
  failed?: unknown[];
  skipped?: unknown[];
}

export type { KillProcInput, KillResult, KillRuleResult };
