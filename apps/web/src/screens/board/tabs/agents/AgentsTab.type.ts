/* @layer renderer-app @kind types */
import type { Agent, ProjectThread, ReconcileFinding } from '../../../../data/agents';
import type { WithId } from '../../../../data/react';
import type { SoftKind } from '../../../../data/kanban';
import type { RunEntry } from '../../../../data/runs';
import type { AgentField } from './model';

interface AgentAnchor {
  selected: string | null;
  open: (id: string) => void;
  close: () => void;
}

interface AgentsTabProps {
  boardKey: string;
  onPurge: (thread: string) => void;
}

interface AgentsView {
  ready: boolean;
  active: WithId<Agent>[];
  done: WithId<Agent>[];
  finishedCount: number;
  lead: WithId<ProjectThread> | null;
  chats: WithId<ProjectThread>[];
  flagSignature: string;
}

interface AgentEdits {
  setField: (id: string, field: AgentField, value: string) => void;
  setItem: (id: string, item: string | null) => void;
  soft: (kind: SoftKind, id: string) => void;
}

interface AgentActions extends AgentEdits {
  stop: (id: string) => void;
  cancelStop: (id: string) => void;
  archiveFinished: (ids: readonly string[]) => void;
}

interface RowState {
  openMessages: ReadonlySet<string>;
  editing: string | null;
  toggleMessage: (id: string) => void;
  startEdit: (id: string) => void;
  stopEdit: () => void;
}

interface AgentRuns {
  all: RunEntry[];
  live: RunEntry[];
  finished: RunEntry[];
  byAgent: ReadonlyMap<string, RunEntry[]>;
}

interface AgentsHeaderProps {
  active: readonly WithId<Agent>[];
  lead: WithId<ProjectThread> | null;
  now: number;
}

interface LeadLineProps {
  lead: WithId<ProjectThread>;
  now: number;
}

interface AgentsBodyProps {
  active: readonly WithId<Agent>[];
  done: readonly WithId<Agent>[];
  showFinished: boolean;
  now: number;
  actions: AgentActions;
  state: RowState;
  runs: AgentRuns;
  onOpen: (id: string) => void;
  onLog: (id: string) => void;
}

interface AgentsFootProps {
  count: number;
  showFinished: boolean;
  canArchive: boolean;
  onToggle: () => void;
  onArchive: () => void;
}

interface ReconcilePanelProps {
  findings: readonly ReconcileFinding[];
}

interface ChatsListProps {
  boardKey: string;
  chats: readonly WithId<ProjectThread>[];
  now: number;
  onPurge: (thread: string) => void;
}

interface ChatRowProps {
  boardKey: string;
  chat: WithId<ProjectThread>;
  now: number;
  onPurge: (thread: string) => void;
}


export type {
  AgentAnchor,
  AgentActions,
  AgentEdits,
  AgentRuns,
  ChatRowProps,
  AgentsBodyProps,
  AgentsFootProps,
  AgentsHeaderProps,
  AgentsTabProps,
  AgentsView,
  ChatsListProps,
  LeadLineProps,
  ReconcilePanelProps,
  RowState,
};
