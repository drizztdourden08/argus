/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
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

interface AgentRowProps {
  id: string;
  agent: Agent;
  now: number;
  actions: AgentActions;
  state: RowState;
  runs: readonly RunEntry[];
  onOpen: (id: string) => void;
  onLog: (id: string) => void;
}

interface AgentCellProps {
  name: string;
  title?: string;
  children?: ReactNode;
}

interface AgentActionsProps {
  id: string;
  agent: Agent;
  actions: AgentActions;
  state: RowState;
  onOpen: (id: string) => void;
}

interface AgentMenuProps {
  id: string;
  agent: Agent;
  actions: AgentActions;
  onOpen: (id: string) => void;
  onEditNote: () => void;
}

interface AgentNameProps {
  id: string;
  agent: Agent;
  now: number;
  runs: readonly RunEntry[];
  onOpen: (id: string) => void;
  onLog: (id: string) => void;
}

interface AgentNoteProps {
  agent: Agent;
}

interface AgentNowProps {
  agent: Agent;
  now: number;
}

interface AgentMessageProps {
  id: string;
  agent: Agent;
  open: boolean;
  onToggle: (id: string) => void;
}

interface AgentFullMessageProps {
  id: string;
  agent: Agent;
  now: number;
  onClose: (id: string) => void;
}

interface AgentNoteEditProps {
  id: string;
  agent: Agent;
  actions: AgentActions;
  onDone: () => void;
}

interface AgentStatusProps {
  agent: Agent;
  now: number;
}

interface AgentStatusTagProps {
  agent: Agent;
}

interface AgentModelProps {
  agent: Agent;
}

interface AgentTimeProps {
  agent: Agent;
  now: number;
}

interface AgentItemProps {
  item: string | null | undefined;
}

interface AgentRunChipsProps {
  runs: readonly RunEntry[];
  onLog: (id: string) => void;
}

interface AgentRunsProps {
  title: string;
  runs: readonly RunEntry[];
  boardKey: string;
  now: number;
  onLog: (id: string) => void;
}

interface AgentRunRowProps {
  run: RunEntry;
  boardKey: string;
  now: number;
  onLog: (id: string) => void;
}

interface AgentDrawerProps {
  boardKey: string;
  id: string | null;
  agent: Agent | null;
  order: readonly string[];
  now: number;
  actions: AgentActions;
  onOpen: (id: string) => void;
  onClose: () => void;
}

interface AgentDrawerBodyProps {
  boardKey: string;
  id: string;
  agent: Agent;
  order: readonly string[];
  now: number;
  actions: AgentActions;
  wide: boolean;
  onWide: () => void;
  onOpen: (id: string) => void;
}

interface AgentDrawerHeadProps {
  boardKey: string;
  id: string;
  agent: Agent;
  now: number;
  actions: AgentActions;
}

interface AgentDrawerToolsProps {
  id: string;
  agent: Agent;
  order: readonly string[];
  actions: AgentActions;
  wide: boolean;
  onWide: () => void;
  onOpen: (id: string) => void;
}

interface AgentItemSelectProps {
  boardKey: string;
  id: string;
  agent: Agent;
  actions: AgentActions;
}

interface AgentFieldProps {
  id: string;
  agent: Agent;
  actions: AgentActions;
}

interface AgentTextFieldProps extends AgentFieldProps {
  field: 'task' | 'note';
}

interface AgentWhereFieldProps extends AgentFieldProps {
  field: 'branch' | 'worktree';
}

interface AgentTimelineProps {
  agent: Agent;
  now: number;
}

interface DrawerSectionProps {
  title: string;
  children: ReactNode;
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

interface ChatMenuProps {
  id: string;
  onPurge: (thread: string) => void;
}

export type {
  AgentAnchor,
  AgentActions,
  AgentActionsProps,
  AgentCellProps,
  AgentDrawerBodyProps,
  AgentDrawerHeadProps,
  AgentDrawerProps,
  AgentDrawerToolsProps,
  AgentEdits,
  AgentFieldProps,
  AgentFullMessageProps,
  AgentItemProps,
  AgentItemSelectProps,
  AgentMenuProps,
  AgentMessageProps,
  AgentModelProps,
  AgentNameProps,
  AgentNoteEditProps,
  AgentNowProps,
  AgentRowProps,
  AgentRunChipsProps,
  AgentRunRowProps,
  AgentRuns,
  AgentRunsProps,
  AgentStatusTagProps,
  AgentTextFieldProps,
  AgentWhereFieldProps,
  AgentNoteProps,
  ChatRowProps,
  AgentsBodyProps,
  AgentsFootProps,
  AgentsHeaderProps,
  AgentsTabProps,
  AgentStatusProps,
  AgentsView,
  AgentTimeProps,
  AgentTimelineProps,
  ChatMenuProps,
  ChatsListProps,
  DrawerSectionProps,
  LeadLineProps,
  ReconcilePanelProps,
  RowState,
};
