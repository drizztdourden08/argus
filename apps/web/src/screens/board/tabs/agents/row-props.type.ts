/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { Agent } from '../../../../data/agents';
import type { RunEntry } from '../../../../data/runs';
import type { AgentActions, RowState } from './AgentsTab.type';

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

interface AgentStopButtonProps {
  id: string;
  agent: Agent;
  actions: AgentActions;
}

interface AgentBranchButtonProps {
  branch: string | null | undefined;
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

export type {
  AgentActionsProps,
  AgentBranchButtonProps,
  AgentStopButtonProps,
  AgentCellProps,
  AgentFullMessageProps,
  AgentItemProps,
  AgentMenuProps,
  AgentMessageProps,
  AgentModelProps,
  AgentNameProps,
  AgentNoteEditProps,
  AgentNoteProps,
  AgentNowProps,
  AgentRowProps,
  AgentRunChipsProps,
  AgentRunRowProps,
  AgentRunsProps,
  AgentStatusProps,
  AgentStatusTagProps,
  AgentTimeProps,
};
