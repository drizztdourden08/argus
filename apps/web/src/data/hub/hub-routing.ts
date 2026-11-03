/* @layer renderer-app @kind logic */
import type { ChangesEvent, SnapshotEvent, StreamStatus, WireChange } from '../stream/stream.type';
import type { HubParts } from './hub-port.type';
import type { HubMessage, MuxFrame, MuxLog, MuxOp, MuxReady, MuxReset, MuxRun, MuxTailEnd } from './hub.type';

class HubRouting {
  private readonly parts: HubParts;
  private readonly scopes = new Map<string, StreamStatus>();
  private link: StreamStatus = 'connecting';

  constructor(parts: HubParts) {
    this.parts = parts;
  }

  frame(frame: MuxFrame): void {
    switch (frame.event) {
      case 'snapshot':
        return this.snapshot(frame.data);
      case 'changes':
        return this.changes(frame.data);
      case 'ready':
        return this.ready(frame.data);
      case 'reset':
        return this.reset(frame.data);
      case 'log':
        return this.log(frame.data);
      case 'run':
        return this.run(frame.data);
      case 'tail-end':
        return this.tailEnd(frame.data);
    }
  }

  status(status: StreamStatus): void {
    this.link = status;
    if (status === 'live') return;
    const next = status === 'idle' ? 'connecting' : status;
    const known = new Set([...this.scopes.keys(), ...this.parts.table.activeCollections().keys()]);
    for (const scope of known) this.setScope(scope, next);
  }

  opError(op: MuxOp): void {
    if (op.op === 'tail') this.tailEnd({ sub: op.sub });
  }

  serve(tabId: string, scope: string, coll: string): void {
    const { cache, port } = this.parts;
    if (!cache.holds(scope, coll)) return;
    const docs = [...cache.collection(scope, coll).docs].map(([id, data]) => ({ id, data }));
    port.post(tabId, { type: 'snapshot', scope, seq: cache.seq(scope), coll, docs });
  }

  follow(tabId: string, scope: string): void {
    this.parts.port.post(tabId, { type: 'status', scope, status: this.statusOf(scope) });
  }

  private snapshot(event: SnapshotEvent): void {
    this.parts.cache.applySnapshot(event);
    this.toTabs(this.parts.table.wantingTabs(event.scope, event.coll), { type: 'snapshot', ...event });
  }

  private changes(event: ChangesEvent): void {
    const { cache, table, port } = this.parts;
    cache.applyChanges(event);
    const perTab = new Map<string, WireChange[]>();
    for (const change of event.changes) {
      if (!cache.holds(event.scope, change.coll)) continue;
      for (const tabId of table.wantingTabs(event.scope, change.coll)) perTab.set(tabId, [...(perTab.get(tabId) ?? []), change]);
    }
    for (const [tabId, changes] of perTab) port.post(tabId, { type: 'changes', ...event, changes });
  }

  private ready(event: MuxReady): void {
    this.parts.cache.markReady(event.scope, event.seq, event.colls);
    this.setScope(event.scope, 'live');
  }

  private reset(event: MuxReset): void {
    const { cache, table, send } = this.parts;
    const dropped = cache.reset(event.scope);
    const holders = new Set(dropped.flatMap((coll) => table.wantingTabs(event.scope, coll)));
    this.toTabs([...holders], { type: 'reset', scope: event.scope });
    this.setScope(event.scope, 'connecting');
    const active = table.activeCollections().get(event.scope);
    if (active) send(cache.resumeOps(new Map([[event.scope, active]])));
  }

  private log({ sub, ...line }: MuxLog): void {
    const { table, port } = this.parts;
    const owner = table.tailOwner(sub);
    if (!owner) return;
    table.tailProgress(sub, line.seq);
    port.post(owner.tabId, { type: 'log', id: owner.id, ...line });
  }

  private run({ sub, run }: MuxRun): void {
    const owner = this.parts.table.tailOwner(sub);
    if (owner) this.parts.port.post(owner.tabId, { type: 'run', id: owner.id, run });
  }

  private tailEnd({ sub }: MuxTailEnd): void {
    const owner = this.parts.table.tailOwner(sub);
    if (owner) this.parts.port.post(owner.tabId, { type: 'tail-end', id: owner.id });
  }

  private statusOf(scope: string): StreamStatus {
    const held = this.scopes.get(scope);
    if (held) return held;
    return this.link === 'unavailable' || this.link === 'unauthorized' ? this.link : 'connecting';
  }

  private setScope(scope: string, status: StreamStatus): void {
    if (this.scopes.get(scope) === status) return;
    this.scopes.set(scope, status);
    this.toTabs(this.parts.table.followingTabs(scope), { type: 'status', scope, status });
  }

  private toTabs(tabIds: readonly string[], message: HubMessage): void {
    for (const tabId of tabIds) this.parts.port.post(tabId, message);
  }
}

export { HubRouting };
