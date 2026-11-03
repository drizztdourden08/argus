/* @layer renderer-app @kind types */
type BundleState = 'draft' | 'ready' | 'closed';

interface Build {
  tag: string;
  status?: 'draft' | 'ready';
  title: string | null;
  note: string | null;
  launcher: string | null;
  cmd: string | null;
  cwd: string | null;
  thread: string | null;
  updatedAt: string;
  closedAt?: string | null;
  closedBy?: string | null;
  closeNote?: string | null;
}

type LauncherStage = 'draft' | 'ready' | 'closed';

interface Launcher {
  name: string;
  cmd: string | null;
  cwd: string | null;
  retiredAt: string | null;
  retiredBy: string | null;
  retireNote: string | null;
  title?: string | null;
  kind?: string | null;
  branch?: string | null;
  commit?: string | null;
  note?: string | null;
  current?: boolean;
  window?: boolean;
  bundle?: string | null;
  by?: string | null;
  createdBy?: string | null;
  updatedAt?: string | null;
  joined?: string | null;
  standalone?: boolean;
  useEvery?: string | null;
  useEveryMs?: number | null;
  lastUsedAt?: string | null;
  keptUntil?: string | null;
  for?: string | null;
  master?: boolean;
  stage?: LauncherStage;
  retiredByItem?: string | null;
  relink?: boolean;
}

interface LaunchDef {
  name: string;
  cmd: string;
  cwd?: string | null;
  launcher?: string;
  bundle?: string;
  hand?: boolean;
  window?: boolean;
  visible?: boolean;
}

interface RunDoc {
  name: string;
  status: 'queued' | 'running' | 'exited' | 'failed' | 'stopped';
  exitCode: number | null;
  queuedAt: string;
  startedAt: string | null;
  endedAt: string | null;
  error: string | null;
  lastLine: string | null;
  stdin?: boolean;
  orphan?: boolean;
  signal?: string | null;
  agent?: string | null;
  origin?: string;
}

interface TestItem {
  title: string;
  thread?: string | null;
  subject?: string | null;
  type?: string | null;
  fields?: Record<string, unknown>;
  deletedAt?: string | null;
  ac?: { text: string; done: boolean }[];
}

export type { Build, BundleState, Launcher, LaunchDef, RunDoc, TestItem };
