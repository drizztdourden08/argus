/* @layer renderer-app @kind types */
import type { RunLogLine, RunRow } from '../runs/runs.type';
import type { HubMessage } from './hub.type';

type ScopeMessage = Extract<HubMessage, { type: 'snapshot' | 'changes' | 'reset' | 'status' }>;

type ScopeRoute = (message: ScopeMessage) => void;

interface TabTailHandlers {
  onLine: (line: RunLogLine) => void;
  onStatus?: (run: RunRow) => void;
  onEnd?: () => void;
}

interface TailEntry {
  key: string;
  rid: string;
  after: number;
  handlers: TabTailHandlers;
}

export type { ScopeMessage, ScopeRoute, TabTailHandlers, TailEntry };
