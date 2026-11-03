/* @layer renderer-app @kind types */
import type { SubscriptionOp } from './subscription-op.type';

interface TailSpec {
  key: string;
  rid: string;
  after: number;
}

interface TabHeld {
  wants: Map<string, Set<string>>;
  presences: Map<string, string>;
  tails: Map<string, TailSpec>;
}

interface TabState {
  wants: { scope: string; coll: string }[];
  presences: { scope: string; thread: string }[];
  tails: ({ id: string } & TailSpec)[];
}

interface TableOptions {
  onExpire: (ops: SubscriptionOp[]) => void;
  lingerMs?: number;
}

interface PresenceHeld {
  scope: string;
  tab: string;
  thread: string;
}

interface TailHeld extends TailSpec {
  sub: string;
}

interface TailOwner {
  tabId: string;
  id: string;
}

export type { PresenceHeld, TabHeld, TableOptions, TabState, TailHeld, TailOwner, TailSpec };
