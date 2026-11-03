/* @layer renderer-app @kind logic */
import { applyChanges } from '../stream/apply-changes';
import { applySnapshot } from '../stream/apply-snapshot';
import { EMPTY_COLLECTION } from '../stream/stream.constants';
import type { ChangesEvent, CollectionState, SnapshotEvent } from '../stream/stream.type';
import type { MuxOp } from './hub.type';
import type { ScopeState } from './scope-cache.type';

const newState = (): ScopeState => ({ collections: new Map(), seq: 0, seqs: new Map(), synced: new Set() });

class ScopeCache {
  private readonly scopes = new Map<string, ScopeState>();

  collection(scope: string, coll: string): CollectionState {
    return this.scopes.get(scope)?.collections.get(coll) ?? EMPTY_COLLECTION;
  }

  holds(scope: string, coll: string): boolean {
    return this.collection(scope, coll).ready;
  }

  isSynced(scope: string, coll: string): boolean {
    return this.scopes.get(scope)?.synced.has(coll) ?? false;
  }

  seq(scope: string): number {
    return this.scopes.get(scope)?.seq ?? 0;
  }

  applySnapshot(event: SnapshotEvent): void {
    const state = this.scopes.get(event.scope) ?? newState();
    state.collections = applySnapshot(state.collections, event);
    state.seqs.set(event.coll, event.seq);
    state.synced.delete(event.coll);
    state.seq = Math.max(state.seq, event.seq);
    this.scopes.set(event.scope, state);
  }

  applyChanges(event: ChangesEvent): string[] {
    const state = this.scopes.get(event.scope);
    if (!state) return [];
    const changes = event.changes.filter((change) => state.collections.has(change.coll));
    const applied = applyChanges(state.collections, { ...event, changes });
    state.collections = applied.collections;
    state.seq = Math.max(state.seq, event.seq);
    for (const coll of state.synced) state.seqs.set(coll, Math.max(state.seqs.get(coll) ?? 0, event.seq));
    return applied.touched;
  }

  markReady(scope: string, seq: number, colls: readonly string[]): void {
    const state = this.scopes.get(scope);
    if (!state) return;
    state.seq = Math.max(state.seq, seq);
    for (const coll of colls.filter((name) => state.collections.has(name))) {
      state.synced.add(coll);
      state.seqs.set(coll, Math.max(state.seqs.get(coll) ?? 0, seq));
    }
  }

  reset(scope: string): string[] {
    const known = [...(this.scopes.get(scope)?.collections.keys() ?? [])];
    this.scopes.delete(scope);
    return known;
  }

  drop(scope: string, colls: readonly string[]): void {
    const state = this.scopes.get(scope);
    if (!state) return;
    const kept = new Map(state.collections);
    for (const coll of colls) {
      kept.delete(coll);
      state.seqs.delete(coll);
      state.synced.delete(coll);
    }
    state.collections = kept;
  }

  resumeOps(wanted: ReadonlyMap<string, readonly string[]>): MuxOp[] {
    return [...wanted].flatMap(([scope, colls]) => this.scopeResume(scope, colls));
  }

  private scopeResume(scope: string, colls: readonly string[]): MuxOp[] {
    const resumable = colls.filter((coll) => this.resumeSeq(scope, coll) > 0);
    const rest = colls.filter((coll) => !resumable.includes(coll));
    const ops: MuxOp[] = [];
    if (resumable.length > 0) {
      const since = Math.min(...resumable.map((coll) => this.resumeSeq(scope, coll)));
      ops.push({ op: 'sub', scope, colls: [...resumable], since });
    }
    if (rest.length > 0) ops.push({ op: 'sub', scope, colls: [...rest] });
    return ops;
  }

  private resumeSeq(scope: string, coll: string): number {
    const state = this.scopes.get(scope);
    return state?.synced.has(coll) ? (state.seqs.get(coll) ?? 0) : 0;
  }
}

export { ScopeCache };
