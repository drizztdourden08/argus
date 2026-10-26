/* @layer renderer-app @kind types */
import type { RunRow } from '../../../data/runs';
import type { StreamStatus } from '../../../data/stream';
import type { Card } from '../../../data/types';
import type { RegistryItem, SendKind, SentItem } from './model';

interface ProjectCardProps {
  card: Card;
  now: number;
  onSend: (key: string) => void;
}

interface CardHeadlineProps {
  card: Card;
  now: number;
}

interface ProjectGridProps {
  cards: Card[];
  onSend: (key: string) => void;
}

interface DashboardTotalsProps {
  cards: Card[];
  status: StreamStatus;
  onSend: (key: string) => void;
}

interface DashboardsTabProps {
  cards: Card[];
  status: StreamStatus;
}

interface SendDown {
  target: string | null;
  name: string;
  kind: SendKind;
  text: string;
  busy: boolean;
  open: (key: string) => void;
  close: () => void;
  setKind: (kind: SendKind) => void;
  setText: (text: string) => void;
  send: () => void;
}

interface SendDownDialogProps {
  send: SendDown;
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
}

export type {
  CardCountersProps, CardHeadlineProps, DashboardsTabProps, DashboardTotalsProps, MasterRowProps, MasterRun, ProjectCardProps, ProjectGridProps, RegistryChipLinkProps,
  RunningNowProps, SendDown, SendDownDialogProps, SentRowProps,
};
