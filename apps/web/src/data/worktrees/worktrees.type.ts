/* @layer renderer-app @kind types */
interface WorktreeKept {
  until?: string | null;
  note?: string | null;
  by?: string | null;
}

interface WorktreeAnswer {
  note: string;
  at: string;
  by?: string | null;
}

interface WorktreeAsk {
  id: string;
  kind: 'delete' | 'check';
  text?: string | null;
  at: string;
  by?: string | null;
  answer: WorktreeAnswer | null;
}

interface WorktreeJob {
  id: string;
  state: string;
  removed?: number;
  total?: number | null;
  error?: string | null;
}

interface LeftoverFacts {
  sizeMB: number | null;
  files: number | null;
  newestAt: string | null;
  ageDays: number | null;
  hasNestedGit?: boolean;
  nestedGit?: string | null;
  nodeModules?: boolean;
  isLink?: boolean;
  partial?: boolean;
  pending?: boolean;
  made?: boolean;
  topEntries?: string[];
}

type WorktreeLinkSource = 'manual' | 'agent' | 'branch' | 'text';

interface Worktree {
  path: string;
  name?: string | null;
  rule?: string | null;
  thread?: string | null;
  branch?: string | null;
  detached?: boolean;
  head?: string | null;
  subject?: string | null;
  lastCommitAt?: string | null;
  lastTouchedAt?: string | null;
  ageDays?: number | null;
  dirty?: number;
  ahead?: number;
  behind?: number;
  merged?: boolean;
  mergedInto?: string | null;
  referenced?: boolean | null;
  unmerged?: number | null;
  unmergedSubjects?: string[];
  locked?: string | boolean | null;
  main?: boolean;
  status: string;
  kept?: WorktreeKept | null;
  request?: WorktreeAsk | null;
  leftover?: LeftoverFacts | null;
  job?: WorktreeJob | null;
  hygiene?: WorktreeHygiene | null;
  item?: string | null;
  linkSource?: WorktreeLinkSource | null;
  unlinked?: boolean;
  agent?: string | null;
  error?: string | null;
}

type HygieneState = 'attached' | 'removing' | 'awaiting-lead' | 'reported' | 'exception';

interface WorktreeHygiene {
  state: HygieneState;
  why?: string | null;
  agent?: string | null;
  thread?: string | null;
  since?: string | null;
  goesAt?: string | null;
  askedAt?: string | null;
  reportedAt?: string | null;
  until?: string | null;
}

interface RemovedWorktree {
  id: string;
  path: string;
  branch?: string | null;
  thread?: string | null;
  at: string;
  text: string;
}

interface HygieneMeta {
  on: boolean;
  debounceMs: number;
  reportAfterMs: number;
  removed: RemovedWorktree[];
}

interface WorktreeRule {
  id: string;
  path: string;
  kind: string;
  include?: string[];
  exclude?: string[];
  invalid?: string | null;
  thread?: string | null;
  note?: string | null;
  staleDays?: number | null;
  exists?: boolean | null;
  candidates?: number | null;
  error?: string | null;
}

interface LeftoverOld {
  days: number;
  count: number;
  sizeMB: number;
}

interface BatchJob {
  id: string;
  state: string;
  name?: string | null;
  path?: string | null;
  error?: string | null;
  finishedAt?: string | null;
}

interface WorktreeRequest {
  id: string;
  by: string;
  at?: string | null;
  mode: string;
  status: string;
  branch?: boolean;
  total: number;
  removed: number;
  refused: number;
  skipped?: number;
  note?: string | null;
  sentences?: string[];
}

interface RequestTarget {
  id: string;
  label: string;
}

interface RequestOutcome {
  id: string;
  label: string;
  ok: boolean;
  text: string;
}

interface RequestRecord {
  id: string;
  by: string;
  at: string;
  mode: string;
  status: string;
  branch: boolean;
  reason?: string | null;
  note?: string | null;
  targets: RequestTarget[];
  results: RequestOutcome[];
  skipped?: { count: number; reasons: string[] } | null;
  startedAt?: string | null;
  finishedAt?: string | null;
}

interface WorktreesMeta {
  name?: string | null;
  scannedAt?: string | null;
  staleDays?: number;
  listed?: number | null;
  runningAgents?: number | null;
  gone?: number;
  tooMany?: boolean;
  rules?: WorktreeRule[];
  leftoverOld?: LeftoverOld | null;
  jobs?: BatchJob[];
  requests?: WorktreeRequest[];
  hygiene?: HygieneMeta | null;
}

interface WorktreeCounter {
  listed: number;
  running: number;
}

interface PortfolioWorktrees {
  counts?: Record<string, WorktreeCounter>;
}

export type {
  BatchJob, PortfolioWorktrees, RemovedWorktree, RequestOutcome, RequestRecord, Worktree, WorktreeHygiene, WorktreeJob, WorktreeRequest, WorktreeRule, WorktreesMeta,
};
