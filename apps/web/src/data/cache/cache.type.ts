/* @layer renderer-app @kind types */
type CacheKind = 'package' | 'electron';

type PlaceKind = 'project' | 'worktree';

interface CacheUser {
  label: string;
  kind: PlaceKind;
}

interface CacheRow {
  kind: CacheKind;
  name: string;
  version: string;
  buildKeys: number;
  users: CacheUser[];
  userCount: number;
  sizeBytes: number;
  lastUsedAt: string;
  inUse: boolean;
  deleteAfter: string | null;
  prunable: number;
  prunableBytes: number;
}

interface CacheTotals {
  versions: number;
  entries: number;
  sizeBytes: number;
  inUseBytes: number;
  unusedBytes: number;
  prunableEntries: number;
  prunableBytes: number;
  abandonedFolders: number;
  abandonedBytes: number;
}

interface DeletedEntry {
  kind: CacheKind;
  name: string;
  version: string;
  buildKey: string;
  sizeBytes: number;
}

interface PruneResult {
  at: string;
  deleted: DeletedEntry[];
  abandonedDeleted: number;
  freedBytes: number;
  refused: string[];
  failed: string[];
}

interface CacheReport {
  root: string;
  exists: boolean;
  graceDays: number;
  rows: CacheRow[];
  totals: CacheTotals;
  problems: string[];
  lastPrune: PruneResult | null;
  shadow: boolean;
}

export type { CacheReport, CacheRow, CacheUser, PruneResult };
