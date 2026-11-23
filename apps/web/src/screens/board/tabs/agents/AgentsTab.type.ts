/* @layer renderer-app @kind types */
import type { Agent, ProjectThread, ReconcileFinding } from '../../../../data/agents';
import type { WithId } from '../../../../data/react';
import type { Item, SoftKind } from '../../../../data/kanban';
import type { RunEntry } from '../../../../data/runs';
import type { TabView } from '../../BoardTabsRow';
import type { AgentField, AgentFilters, AgentGroup, AgentLayout, AgentOrder, AgentStat, ShownAgents } from './model';

interface AgentsTabProps {
  boardKey: string;
}

interface AgentsView {
  ready: boolean;
  active: WithId<Agent>[];
  done: WithId<Agent>[];
  all: WithId<Agent>[];
  shown: ShownAgents;
  items: WithId<Item>[];
  threads: WithId<ProjectThread>[];
  finishedCount: number;
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
  stats: readonly AgentStat[];
}

interface AgentsBodyProps {
  boardKey: string;
  view: AgentsView;
  showFinished: boolean;
  group: AgentGroup;
  layout: AgentLayout;
  now: number;
  actions: AgentActions;
  onOpen: (code: string, tab?: string) => void;
  open: ReadonlySet<string>;
  onRowOpen: (id: string, open: boolean) => void;
}

interface AgentsFootProps {
  canArchive: boolean;
  onArchive: () => void;
}

interface AgentsToolsProps {
  boardKey: string;
  tabView: TabView<AgentGroup, AgentOrder, AgentFilters>;
  view: AgentsView;
  finished: { count: number; shown: boolean; onToggle: () => void };
  layout: { value: AgentLayout; onChange: (layout: AgentLayout) => void };
  expand: { all: boolean; onToggle: () => void };
}

interface ReconcilePanelProps {
  findings: readonly ReconcileFinding[];
}

export type {
  AgentActions,
  AgentEdits,
  AgentRuns,
  AgentsBodyProps,
  AgentsFootProps,
  AgentsHeaderProps,
  AgentsTabProps,
  AgentsToolsProps,
  AgentsView,
  ReconcilePanelProps,
};
