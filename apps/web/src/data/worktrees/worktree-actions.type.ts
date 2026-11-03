/* @layer renderer-app @kind types */
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
  BackupInfo, BatchResult, LeftoverInput, RemoveInput, RemoveResult, WorktreeAskInput,
};
