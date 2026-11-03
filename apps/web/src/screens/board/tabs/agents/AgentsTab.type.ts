/* @layer renderer-app @kind types */
import type { Agent, ProjectThread, ReconcileFinding } from '../../../../data/agents';
import type { WithId } from '../../../../data/react';
import type { Item, SoftKind } from '../../../../data/kanban';
import type { RunEntry } from '../../../../data/runs';
import type { AgentField } from './model';

interface AgentsTabProps {
  boardKey: string;
  onPurge: (thread: string) => void;
}

interface AgentsView {
  ready: boolean;
  active: WithId<Agent>[];
  done: WithId<Agent>[];
  all: WithId<Agent>[];
  items: WithId<Item>[];
  threads: WithId<ProjectThread>[];
  finishedCount: number;
  lead: WithId<ProjectThread> | null;
  chats: WithId<ProjectThread>[];
  flagSignature: string;
}

interface AgentEdits {
  setField: (id: string, field: AgentField, value: string) => void;
  setItems: (id: string, items: readonly string[]) => void;
  setMaster: (id: string, master: boolean) => void;
  soft: (kind: SoftKind, id: string) => void;
}

interface AgentActions extends AgentEdits {
  stop: (id: string) => void;
  cancelStop: (id: string) => void;
  archiveFinished: (ids: readonly string[]) => void;
}

interface AgentRuns {
  all: RunEntry[];
  live: RunEntry[];
  finished: RunEntry[];
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
  boardKey: string;
  view: AgentsView;
  showFinished: boolean;
  now: number;
  actions: AgentActions;
  onOpen: (code: string, tab?: string) => void;
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
};
