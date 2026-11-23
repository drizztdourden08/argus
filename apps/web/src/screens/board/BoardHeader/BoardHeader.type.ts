/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { ProjectThread } from '../../../data/agents';
import type { WithId } from '../../../data/react';
import type { StreamStatus } from '../../../data/stream';
import type { StallDoc } from '../../../data/stalls';
import type { RegistryRow, Wake } from '../../../data/types';

type AlertsTone = 'danger' | 'info';

type Offline = 'unavailable' | 'unauthorized';

interface BoardHeaderProps {
  boardKey: string;
  name: string;
  status: StreamStatus;
  updatedAt: string | null;
  tools: ReactNode;
}

interface BoardTitleProps {
  boardKey: string;
  name: string;
}

interface Rename {
  error: string | null;
  commit: (text: string) => void;
  validate: (text: string) => string | undefined;
}

interface LeadStatusProps {
  lead: WithId<ProjectThread> | null;
}

interface BoardStampProps {
  status: StreamStatus;
  updatedAt: string | null;
}

interface BoardRunningProps {
  runningAgents: number;
}

interface BoardWorktreesProps extends BoardRunningProps {
  listed: number;
}

interface BoardSyncProps {
  boardKey: string;
}

interface BoardAlertsProps {
  boardKey: string;
  status: StreamStatus;
}

interface AlertSectionProps {
  title: string;
  count: number;
  children: ReactNode;
}

interface AlertsLabelProps {
  count: number;
  tone: AlertsTone;
}

interface BoardStallsProps {
  stalls: readonly WithId<StallDoc>[];
  boardKey: string;
}

interface BoardFaultsProps {
  faults: readonly WithId<RegistryRow>[];
}

interface BoardNoticesProps {
  notices: readonly WithId<Wake>[];
}

interface NoticeListProps {
  notices: readonly WithId<Wake>[];
}

type HealthTone = 'success' | 'warning' | 'danger';

interface BoardHealth {
  tone: HealthTone;
  reason: string;
}

interface HealthInput {
  lead: WithId<ProjectThread> | null;
  faults: number;
  offline: boolean;
  updatedAt: string | null;
  now: number;
}

interface BoardHealthView extends BoardHealth {
  lead: WithId<ProjectThread> | null;
  faults: readonly WithId<RegistryRow>[];
  listed: number | undefined;
  runningAgents: number | undefined;
  status: StreamStatus;
  updatedAt: string | null;
}

interface BoardStatusProps {
  boardKey: string;
  health: BoardHealthView;
}

interface AlertsView {
  stalls: WithId<StallDoc>[];
  faults: WithId<RegistryRow>[];
  notices: WithId<Wake>[];
  offline: Offline | null;
  count: number;
  tone: AlertsTone;
}

export type {
  AlertSectionProps,
  AlertsLabelProps,
  AlertsView,
  BoardAlertsProps,
  BoardFaultsProps,
  BoardHeaderProps,
  BoardHealth,
  BoardHealthView,
  BoardStatusProps,
  HealthInput,
  BoardNoticesProps,
  BoardRunningProps,
  BoardStallsProps,
  BoardStampProps,
  BoardTitleProps,
  BoardSyncProps,
  BoardWorktreesProps,
  LeadStatusProps,
  NoticeListProps,
  Rename,
};
