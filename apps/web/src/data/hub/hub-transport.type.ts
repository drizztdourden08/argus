/* @layer renderer-app @kind types */
import type { HealthCheck } from './fallback-tab.type';
import type { HubPort } from './hub-port.type';
import type { HubMessage, TabMessage } from './hub.type';

interface Wire {
  post: (message: unknown) => void;
  listen: (listener: (data: unknown) => void) => void;
  close: () => void;
}

interface LockOptionsLike {
  signal?: AbortSignal;
}

interface LockManagerLike {
  request: (name: string, options: LockOptionsLike, callback: () => unknown) => Promise<unknown>;
}

interface HubEnvelope<M extends { type: string } = TabMessage | HubMessage> {
  tab: string;
  epoch: number;
  msg: M;
}

interface HubLike {
  receive: (tabId: string, message: TabMessage) => void;
  dropTab: (tabId: string) => void;
  dispose: () => void;
}

type CreateHub = (port: HubPort) => HubLike;
type OpenWire = (name: string) => Wire;

interface LeaderDeps {
  tabId: string;
  locks: LockManagerLike;
  openWire: OpenWire;
  createHub: CreateHub;
  now: () => number;
}

interface WorkerHostDeps {
  locks: LockManagerLike | null;
  createHub: CreateHub;
  now: () => number;
}

interface WorkerTabDeps {
  tabId: string;
  wire: Wire;
  locks: LockManagerLike | null;
}

interface InPageDeps {
  tabId: string;
  createHub: CreateHub;
  now: () => number;
}

interface ConnectEnv {
  tabId: string;
  locks: LockManagerLike | null;
  openWorker: (() => Wire) | null;
  openWire: OpenWire | null;
  createHub: CreateHub;
  now: () => number;
  onHide: (listener: () => void) => void;
  health?: HealthCheck;
}

export type {
  ConnectEnv,
  HubEnvelope,
  HubLike,
  InPageDeps,
  LeaderDeps,
  LockManagerLike,
  WorkerHostDeps,
  WorkerTabDeps,
  Wire,
};
