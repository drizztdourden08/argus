/* @layer renderer-app @kind types */
import type { StreamStatus } from '../stream/stream.type';
import type { HubMessage, MuxHandlers, MuxOp, TabMessage } from './hub.type';
import type { ScopeCache } from './scope-cache';
import type { SubscriptionTable } from './subscription-table';

interface HubPort {
  post: (tabId: string, message: HubMessage) => void;
}

interface MuxLink {
  open: () => void;
  close: () => void;
  send: (ops: readonly MuxOp[]) => void;
  getStatus: () => StreamStatus;
}

interface HubOptions {
  port: HubPort;
  connect?: (handlers: MuxHandlers) => MuxLink;
  lingerMs?: number;
}

interface HubParts {
  table: SubscriptionTable;
  cache: ScopeCache;
  port: HubPort;
  send: (ops: readonly MuxOp[]) => void;
}

type LiveTabMessage = Exclude<TabMessage, { type: 'beat' | 'bye' }>;

export type { HubOptions, HubParts, HubPort, LiveTabMessage, MuxLink };
