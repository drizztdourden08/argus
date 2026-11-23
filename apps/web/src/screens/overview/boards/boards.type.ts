/* @layer renderer-app @kind types */
import type { RefObject } from 'react';
import type { RemoteSync } from '../../../data/remote';
import type { RunRow } from '../../../data/runs';
import type { Card } from '../../../data/types';
import type { LeadWords } from './boards.constants';
import type { BoardActionId, BoardLive, BoardProgress, BoardRecord, BoardSignal, MasterLine, SendKind, SentItem } from './model';

type LogOpener = (key: string, run: RunRow) => void;

type BoardHandlers = Readonly<Record<BoardActionId, () => void>>;

interface BoardActions {
  run: MasterRun;
  handlers: BoardHandlers;
}

interface BoardCardProps {
  record: BoardRecord;
  card: Card | null;
  live: BoardLive;
  now: number;
  send: SendDown;
  sync: RemoteSync | null;
  onLog: LogOpener;
}

interface BoardBodyProps {
  record: BoardRecord;
  card: Card | null;
  live: BoardLive;
  now: number;
  run: MasterRun;
  onLog: LogOpener;
  onOpen: (tab: string) => void;
}

interface BoardCardFootProps {
  record: BoardRecord;
  card: Card | null;
  now: number;
  send: SendDown;
  sync: RemoteSync | null;
}

interface BoardLeadProps {
  line: LeadWords;
  agents: string;
}

interface BoardSignalsProps {
  signals: readonly BoardSignal[];
  onOpen: (tab: string) => void;
}

interface BoardProgressProps {
  progress: BoardProgress;
}

interface BoardPlayProps {
  card: Card;
  run: MasterRun;
  now: number;
  onLog: LogOpener;
}

interface LogTarget {
  key: string;
  run: RunRow;
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

interface SendDownPopoverProps {
  send: SendDown;
  owner: string;
  anchorRef: RefObject<HTMLElement | null>;
}

interface MasterRun {
  busy: boolean;
  play: () => void;
  stop: () => void;
}

interface MasterStatusProps {
  line: MasterLine;
  words: string;
  run: RunRow;
  onLog: (run: RunRow) => void;
}

interface RunningNowProps {
  runs: readonly RunRow[];
}

interface SentRowProps {
  entry: SentItem;
}

export type {
  BoardActions, BoardBodyProps, BoardCardFootProps, BoardCardProps, BoardHandlers, BoardLeadProps, BoardPlayProps, BoardProgressProps, BoardSignalsProps, LogOpener, LogTarget, MasterRun, MasterStatusProps, RunningNowProps, SendDown, SendDownPopoverProps, SentRowProps,
};
