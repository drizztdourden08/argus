/* @layer renderer-app @kind types */
import type { LeaderDeps, Wire } from './hub-transport.type';
import type { HubMessage } from './hub.type';

interface LeaderHostOptions {
  deps: LeaderDeps;
  epoch: number;
  bus: Wire;
  deliver: (message: HubMessage) => void;
}

interface Follower {
  wire: Wire;
  unwatch: () => void;
  greeted: boolean;
}

export type { Follower, LeaderHostOptions };
