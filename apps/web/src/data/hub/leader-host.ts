/* @layer renderer-app @kind logic */
import { envelope } from './envelope';
import { envelopeGuard } from './envelope-guard';
import { helloOf } from './hub-hello';
import { EVERY_TAB, INBOX_PREFIX, TAB_TYPES } from './hub-transport.constants';
import type { HubLike, LeaderDeps, Wire } from './hub-transport.type';
import type { HubMessage, TabMessage } from './hub.type';
import { watchTab } from './watch-tab';

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

const isTabEnvelope = envelopeGuard<TabMessage>(TAB_TYPES);

class LeaderHost {
  private readonly options: LeaderHostOptions;
  private readonly hub: HubLike;
  private readonly followers = new Map<string, Follower>();

  constructor(options: LeaderHostOptions) {
    this.options = options;
    this.hub = options.deps.createHub({ post: (tabId, message) => this.post(tabId, message) });
  }

  start(): void {
    const hello = helloOf(this.options.epoch);
    this.options.bus.post(envelope(EVERY_TAB, this.options.epoch, hello));
    this.options.deliver(hello);
  }

  receiveOwn(message: TabMessage): void {
    this.hub.receive(this.options.deps.tabId, message);
  }

  receive(data: unknown): void {
    if (!isTabEnvelope(data) || data.tab === this.options.deps.tabId) return;
    const follower = this.follower(data.tab);
    if (data.epoch !== this.options.epoch) return this.greet(data.tab, follower);
    this.hub.receive(data.tab, data.msg);
    if (data.msg.type === 'bye') this.forget(data.tab);
  }

  stop(): void {
    this.hub.dispose();
    for (const tabId of [...this.followers.keys()]) this.forget(tabId);
  }

  private follower(tabId: string): Follower {
    const known = this.followers.get(tabId);
    if (known) return known;
    const follower: Follower = {
      wire: this.options.deps.openWire(`${INBOX_PREFIX}${tabId}`),
      unwatch: watchTab(this.options.deps.locks, tabId, () => this.gone(tabId)),
      greeted: false,
    };
    this.followers.set(tabId, follower);
    return follower;
  }

  private greet(tabId: string, follower: Follower): void {
    if (follower.greeted) return;
    follower.greeted = true;
    follower.wire.post(envelope(tabId, this.options.epoch, helloOf(this.options.epoch)));
  }

  private post(tabId: string, message: HubMessage): void {
    if (tabId === this.options.deps.tabId) return this.options.deliver(message);
    this.followers.get(tabId)?.wire.post(envelope(tabId, this.options.epoch, message));
  }

  private gone(tabId: string): void {
    this.hub.dropTab(tabId);
    this.forget(tabId);
  }

  private forget(tabId: string): void {
    const follower = this.followers.get(tabId);
    if (!follower) return;
    follower.unwatch();
    follower.wire.close();
    this.followers.delete(tabId);
  }
}

export { LeaderHost };
