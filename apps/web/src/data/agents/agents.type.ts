/* @layer renderer-app @kind types */
import type { PostResult } from '../api';

interface AgentTool {
  name?: string | null;
  summary?: string | null;
}

interface AgentTokens {
  input?: number;
  output?: number;
  cacheRead?: number;
}

interface AgentLive {
  lastMessage?: string | null;
  lastTool?: AgentTool | null;
  activity?: string | null;
  activitySince?: string | null;
  lastActivityAt?: string | null;
  turns?: number;
  toolCalls?: number;
  modelUsed?: string | null;
  tokens?: AgentTokens;
  stalled?: boolean;
  liveNote?: string | null;
}

interface Agent {
  name?: string | null;
  thread?: string | null;
  task?: string | null;
  kind?: string | null;
  item?: string | null;
  model?: string | null;
  modelWhy?: string | null;
  modelUsed?: string | null;
  modelMismatch?: boolean;
  branch?: string | null;
  worktree?: string | null;
  status: string;
  note?: string | null;
  result?: string | null;
  startedAt?: string | null;
  updatedAt?: string | null;
  finishedAt?: string | null;
  rev?: number;
  createdBy?: string | null;
  updatedBy?: string | null;
  claudeId?: string | null;
  live?: AgentLive | null;
  idleSince?: string | null;
  orphanSince?: string | null;
  idMismatch?: boolean;
  lost?: boolean;
  duplicateOf?: string | null;
  reaped?: boolean;
  closedBy?: string | null;
  archivedAt?: string | null;
  deletedAt?: string | null;
}

interface ThreadAgent {
  name?: string | null;
  task?: string | null;
  startedAt?: string | null;
}

interface ProjectThread {
  name: string;
  role: string;
  status: string;
  activity?: string | null;
  note?: string | null;
  session?: string | null;
  subject?: string | null;
  brief?: string | null;
  root?: string | null;
  rev?: number;
  agents?: ThreadAgent[];
  lastSeen?: string | null;
  link?: string | null;
  placeholder?: boolean;
  mergedInto?: string | null;
  archivedAt?: string | null;
  deletedAt?: string | null;
}

interface ReconcileFinding {
  project: string;
  id: string;
  name: string;
  thread: string;
  class: string;
  detail: string;
}

interface ReconcileReport {
  ran: boolean;
  at: string | null;
  findings: ReconcileFinding[];
}

type BulkKind = 'archive' | 'unarchive';

interface BulkSent {
  entryIds: string[];
  results: PostResult[];
}

interface StopSent {
  entryId: string;
  result: PostResult;
}

export type { Agent, AgentTool, BulkKind, BulkSent, ProjectThread, ReconcileFinding, ReconcileReport, StopSent };
