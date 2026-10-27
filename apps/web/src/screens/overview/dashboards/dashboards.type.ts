/* @layer renderer-app @kind types */
import type { IconName } from '@drizztdourden08/tessera';
import type { RunRow } from '../../../data/runs';
import type { StreamStatus } from '../../../data/stream';
import type { Card } from '../../../data/types';
import type { LeadState, MasterLine, RegistryItem, SendKind, SentItem } from './model';

type LogOpener = (key: string, run: RunRow) => void;

interface LogTarget {
  key: string;
  run: RunRow;
}

interface ProjectCardProps {
  card: Card;
  now: number;
  onSend: (key: string) => void;
  onLog: LogOpener;
}

interface CardHeadlineProps {
  card: Card;
  state: LeadState;
}

interface CardStateProps {
  card: Card;
  state: LeadState;
  now: number;
}

interface CardFooterProps {
  card: Card;
  started: boolean;
  onSend: (key: string) => void;
}

interface CounterTileProps {
  icon: IconName;
  value: number;
  label: string;
  bad?: boolean;
}

interface MasterStatusProps {
  line: MasterLine;
  run: RunRow;
  onLog: (run: RunRow) => void;
}

interface ProjectGridProps {
  cards: Card[];
  onSend: (key: string) => void;
  onLog: LogOpener;
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
  onLog: LogOpener;
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
  CardCountersProps, CardFooterProps, CardHeadlineProps, CardStateProps, CounterTileProps, DashboardsTabProps, DashboardTotalsProps, LogTarget, MasterRowProps, MasterRun, MasterStatusProps,
  ProjectCardProps, ProjectGridProps, RegistryChipLinkProps, RunningNowProps, SendDown, SendDownDialogProps, SentRowProps,
};
