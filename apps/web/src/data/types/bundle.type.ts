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

interface Launcher {
  name: string;
  cmd: string | null;
  cwd: string | null;
  retiredAt: string | null;
  retiredBy: string | null;
  retireNote: string | null;
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
}

interface TestItem {
  title: string;
  ac?: { text: string; done: boolean }[];
}

export type { Build, BundleState, Launcher, LaunchDef, RunDoc, TestItem };
