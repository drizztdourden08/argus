/* @layer renderer-app @kind logic */
import type { MuxOp } from './hub.type';
import { SUBSCRIPTION_LINGER_MS } from './subscription-table.constants';
import type { PresenceHeld, TabHeld, TableOptions, TabState, TailHeld, TailOwner, TailSpec } from './subscription-table.type';
import { wantsMap } from './wants-map';

const pairKey = (scope: string, coll: string): string => `${scope}\u0000${coll}`;
const subId = (tabId: string, id: string): string => `${tabId}:${id}`;

class SubscriptionTable {
  private readonly tabs = new Map<string, TabHeld>();
  private readonly active = new Map<string, Set<string>>();
  private readonly timers = new Map<string, ReturnType<typeof setTimeout>>();
  private readonly presenceSent = new Map<string, PresenceHeld>();
  private readonly tailSent = new Map<string, TailHeld>();
  private readonly owners = new Map<string, TailOwner>();
  private readonly lingerMs: number;
  private readonly onExpire: (ops: MuxOp[]) => void;

  constructor(options: TableOptions) {
    this.lingerMs = options.lingerMs ?? SUBSCRIPTION_LINGER_MS;
    this.onExpire = options.onExpire;
  }

  announce(tabId: string, state: TabState): void {
    this.dropTab(tabId);
    const held = this.held(tabId);
    for (const { scope, coll } of state.wants) wantsMap.add(held.wants, scope, coll);
    for (const { scope, thread } of state.presences) held.presences.set(scope, thread);
    for (const { id, ...spec } of state.tails) this.tail(tabId, id, spec);
  }

  want(tabId: string, scope: string, coll: string): void {
    wantsMap.add(this.held(tabId).wants, scope, coll);
  }

  unwant(tabId: string, scope: string, coll: string): void {
    const tab = this.tabs.get(tabId);
    if (tab) wantsMap.remove(tab.wants, scope, coll);
  }

  presence(tabId: string, scope: string, thread: string): void {
    this.held(tabId).presences.set(scope, thread);
  }

  unpresence(tabId: string, scope: string): void {
    this.tabs.get(tabId)?.presences.delete(scope);
  }

  tail(tabId: string, id: string, spec: TailSpec): void {
    this.held(tabId).tails.set(id, { ...spec });
    this.owners.set(subId(tabId, id), { tabId, id });
  }

  untail(tabId: string, id: string): void {
    this.tabs.get(tabId)?.tails.delete(id);
    this.owners.delete(subId(tabId, id));
  }

  tailProgress(sub: string, after: number): void {
    const owner = this.owners.get(sub);
    const targets = [owner && this.tabs.get(owner.tabId)?.tails.get(owner.id), this.tailSent.get(sub)];
    for (const target of targets) if (target && after > target.after) target.after = after;
  }

  dropTab(tabId: string): void {
    for (const [sub, owner] of this.owners) if (owner.tabId === tabId) this.owners.delete(sub);
    this.tabs.delete(tabId);
  }

  flush(): MuxOp[] {
    return [...this.docOps(), ...this.presenceOps(), ...this.tailOps()];
  }

  resumeOps(): MuxOp[] {
    return [
      ...[...this.presenceSent.values()].map((held): MuxOp => ({ op: 'presence', ...held })),
      ...[...this.tailSent.values()].map((held): MuxOp => ({ op: 'tail', ...held })),
    ];
  }

  activeCollections(): Map<string, string[]> {
    return new Map([...this.active].map(([scope, colls]) => [scope, [...colls]]));
  }

  isActive(scope: string, coll: string): boolean {
    return this.active.get(scope)?.has(coll) ?? false;
  }

  forget(scope: string, colls: readonly string[]): void {
    for (const coll of colls) wantsMap.remove(this.active, scope, coll);
  }

  wantingTabs(scope: string, coll: string): string[] {
    return [...this.tabs].filter(([, held]) => held.wants.get(scope)?.has(coll)).map(([tabId]) => tabId);
  }

  followingTabs(scope: string): string[] {
    return [...this.tabs].filter(([, held]) => held.wants.has(scope)).map(([tabId]) => tabId);
  }

  tailOwner(sub: string): TailOwner | undefined {
    return this.owners.get(sub);
  }

  dispose(): void {
    for (const timer of this.timers.values()) clearTimeout(timer);
    this.timers.clear();
  }

  private held(tabId: string): TabHeld {
    const held: TabHeld = this.tabs.get(tabId) ?? { wants: new Map(), presences: new Map(), tails: new Map() };
    this.tabs.set(tabId, held);
    return held;
  }

  private wantedPairs(): Map<string, Set<string>> {
    const wanted = new Map<string, Set<string>>();
    for (const held of this.tabs.values()) {
      for (const [scope, colls] of held.wants) for (const coll of colls) wantsMap.add(wanted, scope, coll);
    }
    return wanted;
  }

  private docOps(): MuxOp[] {
    const wanted = this.wantedPairs();
    const ops: MuxOp[] = [];
    for (const [scope, colls] of wanted) {
      for (const coll of colls) this.cancelLinger(scope, coll);
      const fresh = [...colls].filter((coll) => !this.active.get(scope)?.has(coll));
      for (const coll of fresh) wantsMap.add(this.active, scope, coll);
      if (fresh.length > 0) ops.push({ op: 'sub', scope, colls: fresh });
    }
    this.lingerUnwanted(wanted);
    return ops;
  }

  private lingerUnwanted(wanted: Map<string, Set<string>>): void {
    for (const [scope, colls] of this.active) {
      for (const coll of colls) {
        if (!wanted.get(scope)?.has(coll) && !this.timers.has(pairKey(scope, coll))) this.startLinger(scope, coll);
      }
    }
  }

  private startLinger(scope: string, coll: string): void {
    this.timers.set(pairKey(scope, coll), setTimeout(() => this.expire(scope, coll), this.lingerMs));
  }

  private cancelLinger(scope: string, coll: string): void {
    const key = pairKey(scope, coll);
    clearTimeout(this.timers.get(key));
    this.timers.delete(key);
  }

  private expire(scope: string, coll: string): void {
    this.timers.delete(pairKey(scope, coll));
    if (this.wantedPairs().get(scope)?.has(coll)) return;
    wantsMap.remove(this.active, scope, coll);
    this.onExpire([{ op: 'unsub', scope, colls: [coll] }]);
  }

  private presenceOps(): MuxOp[] {
    const wanted = new Map<string, PresenceHeld>();
    for (const [tab, held] of this.tabs) {
      for (const [scope, thread] of held.presences) wanted.set(pairKey(tab, scope), { scope, tab, thread });
    }
    const ops: MuxOp[] = [];
    for (const [key, sent] of this.presenceSent) {
      if (wanted.has(key)) continue;
      ops.push({ op: 'unpresence', scope: sent.scope, tab: sent.tab });
      this.presenceSent.delete(key);
    }
    for (const [key, held] of wanted) {
      if (this.presenceSent.get(key)?.thread === held.thread) continue;
      ops.push({ op: 'presence', ...held });
      this.presenceSent.set(key, held);
    }
    return ops;
  }

  private tailOps(): MuxOp[] {
    const wanted = new Map<string, TailHeld>();
    for (const [tabId, held] of this.tabs) {
      for (const [id, spec] of held.tails) wanted.set(subId(tabId, id), { sub: subId(tabId, id), ...spec });
    }
    const ops: MuxOp[] = [];
    for (const sub of this.tailSent.keys()) {
      if (wanted.has(sub)) continue;
      ops.push({ op: 'untail', sub });
      this.tailSent.delete(sub);
    }
    for (const [sub, held] of wanted) {
      if (this.tailSent.has(sub)) continue;
      ops.push({ op: 'tail', ...held });
      this.tailSent.set(sub, { ...held });
    }
    return ops;
  }
}

export { SubscriptionTable };
