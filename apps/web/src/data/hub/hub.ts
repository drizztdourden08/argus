/* @layer renderer-app @kind logic */
import { createMuxConnection } from './create-mux-connection';
import type { HubOptions, LiveTabMessage, MuxLink } from './hub-port.type';
import { HubRouting } from './hub-routing';
import { MUX_VERSION } from './hub.constants';
import type { MuxOp, TabMessage } from './hub.type';
import { ScopeCache } from './scope-cache';
import { SubscriptionTable } from './subscription-table';

class Hub {
  private readonly table: SubscriptionTable;
  private readonly cache = new ScopeCache();
  private readonly routing: HubRouting;
  private readonly mux: MuxLink;
  private readonly tabs = new Set<string>();

  constructor(options: HubOptions) {
    this.table = new SubscriptionTable({ lingerMs: options.lingerMs, onExpire: (ops) => this.expire(ops) });
    this.routing = new HubRouting({
      table: this.table,
      cache: this.cache,
      port: options.port,
      send: (ops) => this.mux.send(ops),
    });
    this.mux = (options.connect ?? createMuxConnection)({
      resume: () => [...this.cache.resumeOps(this.table.activeCollections()), ...this.table.resumeOps()],
      onFrame: (frame) => this.routing.frame(frame),
      onStatus: (status) => this.routing.status(status),
      onOpError: (op) => this.routing.opError(op),
    });
  }

  receive(tabId: string, message: TabMessage): void {
    if (message.type === 'beat') return;
    if (message.type === 'bye') return this.dropTab(tabId);
    if (message.type === 'announce' && message.v !== MUX_VERSION) return;
    this.mux.open();
    this.tabs.add(tabId);
    this.apply(tabId, message);
    this.settle();
  }

  dropTab(tabId: string): void {
    this.tabs.delete(tabId);
    this.table.dropTab(tabId);
    this.settle();
  }

  tabCount(): number {
    return this.tabs.size;
  }

  dispose(): void {
    this.table.dispose();
    this.mux.close();
  }

  private apply(tabId: string, message: LiveTabMessage): void {
    switch (message.type) {
      case 'announce':
        return this.announce(tabId, message);
      case 'want':
        return this.want(tabId, message.scope, message.coll);
      case 'unwant':
        return this.table.unwant(tabId, message.scope, message.coll);
      case 'presence':
        return this.table.presence(tabId, message.scope, message.thread);
      case 'unpresence':
        return this.table.unpresence(tabId, message.scope);
      case 'tail':
        return this.table.tail(tabId, message.id, { key: message.key, rid: message.rid, after: message.after });
      case 'untail':
        return this.table.untail(tabId, message.id);
    }
  }

  private announce(tabId: string, message: Extract<TabMessage, { type: 'announce' }>): void {
    const { wants, presences, tails } = message;
    this.table.announce(tabId, { wants, presences, tails });
    for (const scope of new Set(wants.map((want) => want.scope))) this.routing.follow(tabId, scope);
    for (const { scope, coll } of wants) this.routing.serve(tabId, scope, coll);
  }

  private want(tabId: string, scope: string, coll: string): void {
    if (!this.table.followingTabs(scope).includes(tabId)) this.routing.follow(tabId, scope);
    this.table.want(tabId, scope, coll);
    this.routing.serve(tabId, scope, coll);
  }

  private settle(): void {
    const ops = this.table.flush();
    if (ops.length > 0) this.mux.send(ops);
  }

  private expire(ops: MuxOp[]): void {
    for (const op of ops) if (op.op === 'unsub') this.cache.drop(op.scope, op.colls ?? []);
    this.mux.send(ops);
  }
}

export { Hub };
