/* @layer renderer-app @kind types */
import type { Agent, ProjectThread } from '../../../../data/agents';
import type { Item } from '../../../../data/kanban';
import type { WithId } from '../../../../data/react';
import type { RunEntry } from '../../../../data/runs';
import type { AgentActions, AgentsView } from './AgentsTab.type';
import type { AgentGroup } from './model';

type OpenRecord = (code: string, tab?: string) => void;

interface AgentRowProps {
  boardKey: string;
  id: string;
  agent: Agent;
  threads: AgentsView['threads'];
  now: number;
  actions: AgentActions;
  onOpen: OpenRecord;
}

interface AgentTrailingProps {
  id: string;
  agent: Agent;
  now: number;
  actions: AgentActions;
}

interface AgentStopButtonProps {
  id: string;
  agent: Agent;
  actions: AgentActions;
}

interface AgentNowProps {
  agent: Agent;
  now: number;
}

interface AgentLiveNoteProps {
  agent: Agent;
}

interface AgentSessionProps {
  agent: Agent;
}

interface AgentReplyProps {
  title: string;
  text: string | null | undefined;
  empty: string;
}

interface AgentStatusTagProps {
  agent: Agent;
}

interface AgentModelProps {
  agent: Agent;
}

interface ItemHeaderProps {
  boardKey: string;
  itemId: string;
  items: readonly WithId<Item>[];
  threads: readonly WithId<ProjectThread>[];
  onOpen: OpenRecord;
}

interface ItemGroupProps extends Omit<AgentRowProps, 'id' | 'agent'> {
  itemId: string;
  items: readonly WithId<Item>[];
  agents: readonly WithId<Agent>[];
}

interface AgentGroupListProps extends Omit<AgentRowProps, 'id' | 'agent'> {
  label: string;
  agents: readonly WithId<Agent>[];
  items: readonly WithId<Item>[];
}

interface AgentSectionProps extends AgentGroupListProps {
  title: string;
  group: AgentGroup;
}

interface AgentKeyGroupsProps extends Omit<AgentGroupListProps, 'items'> {
  group: AgentGroup;
}

interface AgentRunsProps {
  title: string;
  runs: readonly RunEntry[];
  boardKey: string;
  now: number;
}

interface AgentRunRowProps {
  run: RunEntry;
  boardKey: string;
  now: number;
}

export type {
  AgentGroupListProps,
  AgentKeyGroupsProps,
  AgentLiveNoteProps,
  AgentModelProps,
  AgentNowProps,
  AgentReplyProps,
  AgentRowProps,
  AgentRunRowProps,
  AgentRunsProps,
  AgentSectionProps,
  AgentSessionProps,
  AgentStatusTagProps,
  AgentStopButtonProps,
  AgentTrailingProps,
  ItemGroupProps,
  ItemHeaderProps,
  OpenRecord,
};
