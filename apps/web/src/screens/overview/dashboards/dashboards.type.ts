/* @layer renderer-app @kind types */
import type { RunRow } from '../../../data/runs';
import type { StreamStatus } from '../../../data/stream';
import type { Card } from '../../../data/types';
import type { RegistryItem, SentItem } from './model';

interface ProjectCardProps {
  card: Card;
  now: number;
}

interface ProjectGridProps {
  cards: Card[];
}

interface DashboardTotalsProps {
  cards: Card[];
  status: StreamStatus;
}

interface DashboardsTabProps {
  cards: Card[];
  status: StreamStatus;
}

interface MasterRun {
  busy: boolean;
  play: () => void;
  stop: () => void;
}

interface MasterRowProps {
  card: Card;
  now: number;
}

interface RunningNowProps {
  runs: readonly RunRow[];
}

interface CardCountersProps {
  card: Card;
}

interface RegistryChipLinkProps {
  row: RegistryItem;
}

interface SentRowProps {
  entry: SentItem;
  now: number;
}

export type {
  CardCountersProps, DashboardsTabProps, DashboardTotalsProps, MasterRowProps, MasterRun, ProjectCardProps, ProjectGridProps, RegistryChipLinkProps,
  RunningNowProps, SentRowProps,
};
