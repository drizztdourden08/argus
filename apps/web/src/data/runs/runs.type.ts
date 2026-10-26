/* @layer renderer-app @kind types */
import type { WithId } from '../react';
import type { RunDoc } from '../types';

type RunKind = 'build' | 'test' | 'launch' | 'task';
type RunStatus = 'queued' | 'running' | 'exited' | 'failed' | 'stopped';
type RunTone = 'warning' | 'success' | 'info' | 'neutral' | 'danger';
type RunStream = 'out' | 'err' | 'sys';

type RunEntry = WithId<RunDoc>;

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

interface RunLine {
  tone: RunTone;
  label: string;
  since: string | null;
  last: string;
}

interface RunLogLine {
  seq: number;
  stream: RunStream;
  at: string;
  text: string;
}

interface RunLogPage {
  lines: RunLogLine[];
  next: number;
  dropped: number;
}

interface RunLogQuery {
  tail?: number;
  after?: number;
  limit?: number;
  stream?: RunStream;
}

interface RunAttachment {
  id: string;
  url: string;
  caption: string;
  mime: string;
  bytes: number;
  n: number;
  by: string;
  at: string;
}

interface RunTailHandlers {
  onLine: (line: RunLogLine) => void;
  onStatus?: (run: RunRow) => void;
}

interface RunLogOptions {
  limit: number;
  live: boolean;
  enabled: boolean;
}

interface RunLogView {
  lines: RunLogLine[];
  clear: () => void;
}

interface LineBuffer extends RunLogView {
  push: (incoming: RunLogLine[]) => void;
  note: (text: string) => void;
  reset: () => void;
}

export type {
  LineBuffer, RunAttachment, RunEntry, RunFilter, RunLine, RunLogLine, RunLogOptions, RunLogPage, RunLogQuery, RunLogView, RunRow,
  RunStatus, RunTailHandlers, RunTone, StartRunInput,
};
