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
  error?: string | null;
}

interface WorktreeHygiene {
  state: string;
  why?: string | null;
  agent?: string | null;
  thread?: string | null;
  since?: string | null;
  goesAt?: string | null;
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
  graceMs: number;
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

interface WorktreeAskInput {
  id: string;
  path: string;
  kind: 'delete' | 'check';
  thread: string;
  message: string;
}

interface RemoveInput {
  id: string;
  branch: boolean;
  discard: boolean;
  head: string | null;
  dirty: number;
}

interface LeftoverInput {
  id: string;
  newestAt: string | null;
  confirmYoung: boolean;
}

interface BackupInfo {
  dir?: string;
  bundle?: string;
  patch?: string;
  restore?: string[];
}

interface RemoveResult {
  prunedOnly?: boolean;
  branch?: string | null;
  deletedBranch?: boolean;
  branchNote?: string | null;
  branchWhy?: string | null;
  backup?: BackupInfo | null;
  jobId?: string | null;
}

interface SkippedLeftover {
  id: string;
  path?: string;
  reason: string;
}

interface BatchResult {
  started?: number;
  skipped?: SkippedLeftover[];
}

export type {
  BackupInfo, BatchJob, BatchResult, LeftoverInput, PortfolioWorktrees, RemovedWorktree, RemoveInput, RemoveResult, Worktree, WorktreeAskInput, WorktreeHygiene,
  WorktreeJob, WorktreeLinkSource, WorktreeRequest, WorktreeRule, WorktreesMeta,
};
