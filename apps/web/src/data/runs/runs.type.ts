/* @layer renderer-app @kind types */
type RunKind = 'build' | 'test' | 'launch' | 'task';
type RunStatus = 'queued' | 'running' | 'exited' | 'failed' | 'stopped';

interface RunRow {
  id: string;
  scope: string;
  name: string;
  kind: RunKind;
  status: RunStatus;
  exitCode: number | null;
  signal: string | null;
  pid: number | null;
  visible: boolean;
  stdin: boolean;
  origin: 'page' | 'mcp' | 'cli';
  by: string;
  agent: string | null;
  reason: string | null;
  queuedAt: string;
  startedAt: string | null;
  endedAt: string | null;
  logBytes: number;
  logDropped: number;
  orphan: boolean;
  error: string | null;
  lastLine: string | null;
  cmd: string;
  cwd: string;
}

interface StartRunInput {
  name: string;
  visible?: boolean;
  agent?: string;
  reason?: string;
}

interface RunFilter {
  status?: RunStatus;
  name?: string;
  limit?: number;
}

export type { RunFilter, RunRow, StartRunInput };
