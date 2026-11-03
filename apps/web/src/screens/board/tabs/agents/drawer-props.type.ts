/* @layer renderer-app @kind types */
import type { Agent } from '../../../../data/agents';
import type { AgentActions } from './AgentsTab.type';

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

export type { AgentFieldProps, AgentItemSelectProps, AgentTextFieldProps, AgentTimelineProps, AgentWhereFieldProps };
