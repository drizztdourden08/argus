/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { Agent, ProjectThread, ReconcileFinding } from '../../../../data/agents';
import type { WithId } from '../../../../data/react';
import type { AgentFilter } from './model';

interface AgentsTabProps {
  boardKey: string;
}

interface AgentsView {
  ready: boolean;
  active: WithId<Agent>[];
  done: WithId<Agent>[];
  lead: WithId<ProjectThread> | null;
  chats: WithId<ProjectThread>[];
  flagSignature: string;
}

interface AgentActions {
  stop: (id: string) => void;
  cancelStop: (id: string) => void;
  archiveFinished: (ids: readonly string[]) => void;
}

interface AgentsHeaderProps {
  active: readonly WithId<Agent>[];
  lead: WithId<ProjectThread> | null;
}

interface LeadLineProps {
  lead: WithId<ProjectThread>;
}

interface AgentsToolbarProps {
  query: string;
  onQuery: (query: string) => void;
  filter: AgentFilter;
  onFilter: (filter: AgentFilter) => void;
  runningCount: number;
  doneCount: number;
  onArchive: (() => void) | null;
}

interface ReconcilePanelProps {
  findings: readonly ReconcileFinding[];
}

interface AgentRowProps {
  id: string;
  agent: Agent;
  now: number;
  actions: AgentActions;
  onOpen: (id: string) => void;
}

interface AgentListProps {
  rows: readonly WithId<Agent>[];
  empty: string;
  now: number;
  actions: AgentActions;
  onOpen: (id: string) => void;
}

interface AgentActionsProps {
  id: string;
  agent: Agent;
  actions: AgentActions;
}

interface AgentNameProps {
  id: string;
  agent: Agent;
  now: number;
}

interface AgentDetailLineProps {
  agent: Agent;
}

interface AgentStatusProps {
  agent: Agent;
}

interface AgentModelProps {
  agent: Agent;
}

interface AgentTimeProps {
  agent: Agent;
  now: number;
}

interface AgentDrawerProps {
  id: string | null;
  agent: Agent | null;
  now: number;
  actions: AgentActions;
  onClose: () => void;
}

interface AgentDrawerBodyProps {
  agent: Agent;
  now: number;
}

interface AgentFactsProps {
  agent: Agent;
  now: number;
}

interface DrawerSectionProps {
  title: string;
  children: ReactNode;
}

interface ChatsListProps {
  chats: readonly WithId<ProjectThread>[];
}

export type {
  AgentActions,
  AgentActionsProps,
  AgentDetailLineProps,
  AgentDrawerBodyProps,
  AgentDrawerProps,
  AgentFactsProps,
  AgentListProps,
  AgentModelProps,
  AgentNameProps,
  AgentRowProps,
  AgentStatusProps,
  AgentsHeaderProps,
  AgentsTabProps,
  AgentsToolbarProps,
  AgentsView,
  AgentTimeProps,
  ChatsListProps,
  DrawerSectionProps,
  LeadLineProps,
  ReconcilePanelProps,
};
