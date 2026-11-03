/* @layer renderer-app @kind types */
import type { RunLogLine, RunRow } from '../runs/runs.type';
import type { ChangesEvent, SnapshotEvent, SourceLike, StreamStatus } from '../stream/stream.type';
import type { MUX_EVENTS } from './hub.constants';

interface MuxSubOp {
  op: 'sub';
  scope: string;
  colls: string[];
  since?: number;
}

interface MuxUnsubOp {
  op: 'unsub';
  scope: string;
  colls?: string[];
}

interface MuxPresenceOp {
  op: 'presence';
  scope: string;
  thread: string;
  tab: string;
}

interface MuxUnpresenceOp {
  op: 'unpresence';
  scope: string;
  tab: string;
}

interface MuxTailOp {
  op: 'tail';
  sub: string;
  key: string;
  rid: string;
  after: number;
}

interface MuxUntailOp {
  op: 'untail';
  sub: string;
}

type MuxOp = MuxSubOp | MuxUnsubOp | MuxPresenceOp | MuxUnpresenceOp | MuxTailOp | MuxUntailOp;

interface MuxOpError {
  code: string;
  message: string;
}

type MuxOpResult = { ok: true } | { error: MuxOpError };

interface MuxHello {
  conn: string;
  v: number;
}

interface MuxReady {
  scope: string;
  seq: number;
  colls: string[];
}

interface MuxReset {
  scope: string;
  reason: string;
}

interface MuxLog extends RunLogLine {
  sub: string;
}

interface MuxRun {
  sub: string;
  run: RunRow;
}

interface MuxTailEnd {
  sub: string;
}

type MuxFrame =
  | { event: 'snapshot'; data: SnapshotEvent }
  | { event: 'changes'; data: ChangesEvent }
  | { event: 'ready'; data: MuxReady }
  | { event: 'reset'; data: MuxReset }
  | { event: 'log'; data: MuxLog }
  | { event: 'run'; data: MuxRun }
  | { event: 'tail-end'; data: MuxTailEnd };

type MuxEventName = (typeof MUX_EVENTS)[number];

interface MuxPostResult {
  status: number;
  results: MuxOpResult[];
}

interface MuxTransport {
  openSource: (url: string) => SourceLike;
  post: (conn: string, ops: MuxOp[]) => Promise<MuxPostResult>;
  health: () => Promise<number | null>;
}

interface MuxHandlers {
  resume: () => readonly MuxOp[];
  onFrame: (frame: MuxFrame) => void;
  onStatus: (status: StreamStatus) => void;
  onOpError: (op: MuxOp, error: MuxOpError) => void;
}

type MuxOptions = MuxHandlers & MuxTransport;

interface OutboxHandlers {
  post: MuxTransport['post'];
  onDelivered: () => void;
  onGone: () => void;
  onFailed: () => void;
  onOpError: MuxHandlers['onOpError'];
}

interface TabWant {
  scope: string;
  coll: string;
}

interface TabPresenceWant {
  scope: string;
  thread: string;
}

interface TabTailWant {
  id: string;
  key: string;
  rid: string;
  after: number;
}

type TabMessage =
  | { type: 'announce'; tabId: string; v: number; wants: TabWant[]; presences: TabPresenceWant[]; tails: TabTailWant[] }
  | ({ type: 'want' } & TabWant)
  | ({ type: 'unwant' } & TabWant)
  | ({ type: 'presence' } & TabPresenceWant)
  | { type: 'unpresence'; scope: string }
  | ({ type: 'tail' } & TabTailWant)
  | { type: 'untail'; id: string }
  | { type: 'bye' }
  | { type: 'beat' };

type HubMessage =
  | { type: 'hub-hello'; epoch: number; v: number }
  | ({ type: 'snapshot' } & SnapshotEvent)
  | ({ type: 'changes' } & ChangesEvent)
  | { type: 'reset'; scope: string }
  | { type: 'status'; scope: string; status: StreamStatus }
  | ({ type: 'log'; id: string } & RunLogLine)
  | { type: 'run'; id: string; run: RunRow }
  | { type: 'tail-end'; id: string };

export type {
  HubMessage,
  MuxEventName,
  MuxFrame,
  MuxHandlers,
  MuxHello,
  MuxLog,
  MuxOp,
  MuxOpResult,
  MuxOptions,
  MuxPostResult,
  MuxReady,
  MuxReset,
  MuxRun,
  MuxTailEnd,
  OutboxHandlers,
  TabMessage,
};
