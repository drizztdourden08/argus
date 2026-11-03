/* @layer renderer-app @kind logic */
import { envelope } from './envelope';
import { envelopeGuard } from './envelope-guard';
import { helloOf } from './hub-hello';
import { BEAT_MS, DEAD_AFTER_MS, TAB_TYPES, WORKER_IDLE_MS } from './hub-transport.constants';
import type { HubLike, WorkerHostDeps, Wire } from './hub-transport.type';
import type { TabMessage } from './hub.type';
import { watchTab } from './watch-tab';
import type { Member } from './worker-host.type';

const isTabEnvelope = envelopeGuard<TabMessage>(TAB_TYPES);

class WorkerHost {
  private readonly deps: WorkerHostDeps;
  private readonly members = new Map<string, Member>();
  private hub: HubLike | null = null;
  private epoch = 0;
  private sweeper: ReturnType<typeof setInterval> | null = null;
  private idle: ReturnType<typeof setTimeout> | null = null;

  constructor(deps: WorkerHostDeps) {
    this.deps = deps;
  }

  connect(wire: Wire): void {
    wire.listen((data) => this.receive(wire, data));
  }

  private receive(wire: Wire, data: unknown): void {
    if (!isTabEnvelope(data)) return;
    const member = this.members.get(data.tab) ?? this.join(data.tab, wire);
    member.seen = this.deps.now();
    this.hubOf().receive(data.tab, data.msg);
    if (data.msg.type === 'bye') this.leave(data.tab);
  }

  private hubOf(): HubLike {
    if (this.hub) return this.hub;
    this.epoch = this.deps.now();
    this.hub = this.deps.createHub({
      post: (tabId, message) => this.members.get(tabId)?.wire.post(envelope(tabId, this.epoch, message)),
    });
    return this.hub;
  }

  private join(tabId: string, wire: Wire): Member {
    this.cancelIdle();
    this.hubOf();
    const member: Member = { wire, seen: this.deps.now(), unwatch: this.watch(tabId) };
    this.members.set(tabId, member);
    wire.post(envelope(tabId, this.epoch, helloOf(this.epoch)));
    return member;
  }

  private watch(tabId: string): () => void {
    const { locks } = this.deps;
    if (locks) return watchTab(locks, tabId, () => this.gone(tabId));
    this.sweeper ??= setInterval(() => this.sweep(), BEAT_MS);
    return () => undefined;
  }

  private sweep(): void {
    const now = this.deps.now();
    for (const [tabId, member] of [...this.members]) if (now - member.seen > DEAD_AFTER_MS) this.gone(tabId);
  }

  private gone(tabId: string): void {
    this.hub?.dropTab(tabId);
    this.leave(tabId);
  }

  private leave(tabId: string): void {
    const member = this.members.get(tabId);
    if (!member) return;
    member.unwatch();
    member.wire.close();
    this.members.delete(tabId);
    if (this.members.size === 0) this.rest();
  }

  private rest(): void {
    if (this.sweeper) clearInterval(this.sweeper);
    this.sweeper = null;
    this.idle = setTimeout(() => this.dispose(), WORKER_IDLE_MS);
  }

  private dispose(): void {
    this.idle = null;
    if (this.members.size > 0) return;
    this.hub?.dispose();
    this.hub = null;
  }

  private cancelIdle(): void {
    if (this.idle) clearTimeout(this.idle);
    this.idle = null;
  }
}

export { WorkerHost };
