/* @layer renderer-app @kind types */
import type { SignalItem } from '@drizztdourden08/tessera';
import type { RemoteSync } from '../../../data/remote';
import type { RunRow } from '../../../data/runs';
import type { Card } from '../../../data/types';
import type { MasterRun, SendDown } from '../dashboards/dashboards.type';
import type { BoardActionId, BoardLive, BoardRecord } from './model';

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

export type { BoardActions, BoardCardFootProps, BoardCardProps, BoardHandlers, BoardPlayProps, LogOpener, StripTone };
