/* @layer renderer-app @kind logic */
import { MUX_VERSION } from './hub.constants';
import type { HubMessage } from './hub.type';
import type { TabHandle } from './tab-handle.type';
import type { ScopeRoute, TabTailHandlers, TailEntry, TailMessage } from './tab-link.type';

class TabLink {
  readonly tabId: string;
  private readonly handle: TabHandle;
  private readonly routes = new Map<string, ScopeRoute>();
  private readonly wants = new Map<string, Set<string>>();
  private readonly presences = new Map<string, string>();
  private readonly tails = new Map<string, TailEntry>();
  private tailCount = 0;

  constructor(handle: TabHandle) {
    this.handle = handle;
    this.tabId = handle.tabId;
    handle.onMessage((message) => this.receive(message));
  }

  route(scope: string, route: ScopeRoute): void {
    this.routes.set(scope, route);
  }

  want(scope: string, coll: string): void {
    const colls = this.wants.get(scope) ?? new Set<string>();
    if (colls.has(coll)) return;
    this.wants.set(scope, colls.add(coll));
    this.handle.send({ type: 'want', scope, coll });
  }

  unwant(scope: string, coll: string): void {
    const colls = this.wants.get(scope);
    if (!colls?.delete(coll)) return;
    if (colls.size === 0) this.wants.delete(scope);
    this.handle.send({ type: 'unwant', scope, coll });
  }

  presence(scope: string, thread: string): () => void {
    this.presences.set(scope, thread);
    this.handle.send({ type: 'presence', scope, thread });
    return () => {
      if (this.presences.get(scope) !== thread) return;
      this.presences.delete(scope);
      this.handle.send({ type: 'unpresence', scope });
    };
  }

  tail(key: string, rid: string, after: number, handlers: TabTailHandlers): () => void {
    this.tailCount += 1;
    const id = `t${this.tailCount}`;
    this.tails.set(id, { key, rid, after, handlers });
    this.handle.send({ type: 'tail', id, key, rid, after });
    return () => {
      this.untail(id);
    };
  }

  private untail(id: string): TailEntry | undefined {
    const entry = this.tails.get(id);
    if (!entry) return undefined;
    this.tails.delete(id);
    this.handle.send({ type: 'untail', id });
    return entry;
  }

  private receive(message: HubMessage): void {
    switch (message.type) {
      case 'hub-hello':
        return this.announce();
      case 'log':
      case 'run':
      case 'tail-end':
        return this.receiveTail(message);
      case 'snapshot':
      case 'changes':
      case 'reset':
      case 'status':
        return this.routes.get(message.scope)?.(message);
    }
  }

  private receiveTail(message: TailMessage): void {
    switch (message.type) {
      case 'log':
        return this.onLog(message);
      case 'run':
        return this.tails.get(message.id)?.handlers.onStatus?.(message.run);
      case 'tail-end':
        return this.untail(message.id)?.handlers.onEnd?.();
    }
  }

  private onLog(message: Extract<TailMessage, { type: 'log' }>): void {
    const entry = this.tails.get(message.id);
    if (!entry || message.seq <= entry.after) return;
    entry.after = message.seq;
    const { seq, stream, at, text } = message;
    entry.handlers.onLine({ seq, stream, at, text });
  }

  private announce(): void {
    this.handle.send({
      type: 'announce',
      tabId: this.tabId,
      v: MUX_VERSION,
      wants: [...this.wants].flatMap(([scope, colls]) => [...colls].map((coll) => ({ scope, coll }))),
      presences: [...this.presences].map(([scope, thread]) => ({ scope, thread })),
      tails: [...this.tails].map(([id, { key, rid, after }]) => ({ id, key, rid, after })),
    });
  }
}

export { TabLink };
