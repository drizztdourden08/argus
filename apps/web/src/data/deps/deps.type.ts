/* @layer renderer-app @kind types */
type DepState = 'opening' | 'open' | 'done' | 'dropped' | 'blocked' | 'gone' | 'cancelled';

type DepColor = 'info' | 'warning' | 'success' | 'neutral';

interface DepEnd {
  project: string;
  item: string | null;
  thread?: string | null;
}

interface DepHistoryRow {
  seq: number;
  at: string;
  by: string;
  from: DepState | null;
  to: DepState;
  reason: string | null;
}

interface DepDoc {
  id: string;
  rev?: number;
  createdAt?: string;
  createdBy?: string;
  updatedAt?: string;
  state: DepState;
  from: DepEnd;
  to: DepEnd;
  title: string;
  reason: string | null;
  openedAt: string | null;
  resolvedAt: string | null;
  history: DepHistoryRow[];
}

interface WaitsOn {
  dep: string;
  project: string;
  item: string;
  title: string;
  state: DepState;
  since: string;
  reason: string | null;
}

interface RequestedBy {
  dep: string;
  project: string;
  item: string;
  thread: string;
  title: string;
  state: DepState;
}

interface DepAnswer {
  dep: DepDoc;
  text: string;
}

export type { DepAnswer, DepColor, DepDoc, DepHistoryRow, DepState, RequestedBy, WaitsOn };
