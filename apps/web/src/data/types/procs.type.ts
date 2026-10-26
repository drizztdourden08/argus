/* @layer renderer-app @kind types */
interface ProcDoc {
  pid?: number;
  ppid?: number;
  name?: string;
  cmd?: string | null;
  hint?: string | null;
  rule?: string | null;
  project?: string | null;
  memMB?: number | null;
  cpuPct?: number | null;
  startedAt?: string;
  ageMin?: number | null;
  idleMin?: number | null;
  responding?: boolean | null;
  parentAlive?: boolean;
  status?: string;
  flags?: string[];
  protected?: boolean;
  protectedWhy?: string | null;
  killable?: boolean;
  since?: string;
}

interface ProcRuleMatch {
  name?: string;
  cmd?: string;
  path?: string;
}

interface ProcRule {
  name: string;
  match?: ProcRuleMatch;
  minMemMB?: number;
  onlyFlagged?: boolean;
  maxMemMB?: number;
  maxGroupMemMB?: number;
  maxAgeMin?: number;
  idleMin?: number;
  orphan?: boolean;
  hung?: boolean;
  killable?: boolean;
  note?: string;
}

interface RuleGroup {
  rule: string;
  count?: number;
  matched?: number;
  memMB?: number;
  maxGroupMemMB?: number;
  over?: boolean;
}

interface ProcsMeta {
  sampledAt?: string;
  sampleMs?: number;
  count?: number;
  totalMemMB?: number;
  byRule?: RuleGroup[];
  rules?: ProcRule[];
  rulesFile?: string;
  rulesError?: string | null;
  platform?: string;
  error?: string | null;
  tookMs?: number;
}

export type { ProcDoc, ProcRule, ProcRuleMatch, ProcsMeta, RuleGroup };
