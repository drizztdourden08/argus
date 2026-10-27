/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { Agent } from '../../../../data/agents';
import type { AgentActions } from './AgentsTab.type';

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
  now: number;
  actions: AgentActions;
}

interface AgentEyebrowProps {
  id: string;
  agent: Agent;
  now: number;
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

export type {
  AgentDrawerBodyProps,
  AgentDrawerHeadProps,
  AgentDrawerProps,
  AgentDrawerToolsProps,
  AgentEyebrowProps,
  AgentFieldProps,
  AgentItemSelectProps,
  AgentTextFieldProps,
  AgentTimelineProps,
  AgentWhereFieldProps,
  DrawerSectionProps,
};
