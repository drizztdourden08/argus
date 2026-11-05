/* @layer renderer-app @kind types */
import type { SignalItem } from '@drizztdourden08/tessera';
import type { RefObject } from 'react';
import type { RemoteSync } from '../../../data/remote';
import type { RunRow } from '../../../data/runs';
import type { Card } from '../../../data/types';
import type { BoardActionId, BoardLive, BoardRecord, MasterLine, SendKind, SentItem } from './model';

type StripTone = SignalItem['tone'];

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

interface BoardCardFootProps {
  record: BoardRecord;
  card: Card | null;
  now: number;
  send: SendDown;
  run: MasterRun;
  sync: RemoteSync | null;
  onLog: LogOpener;
}

interface BoardPlayProps {
  card: Card;
  run: MasterRun;
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
  BoardActions, BoardCardFootProps, BoardCardProps, BoardHandlers, BoardPlayProps, LogOpener, LogTarget, MasterRun, MasterStatusProps, RunningNowProps, SendDown, SendDownPopoverProps, SentRowProps, StripTone,
};
