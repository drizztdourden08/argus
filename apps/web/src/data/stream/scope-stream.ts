/* @layer renderer-app @kind logic */
import type { ScopeMessage } from '../hub/tab-link.type';
import type { TabLink } from '../hub/tab-link';
import { applyChanges } from './apply-changes';
import { applySnapshot } from './apply-snapshot';
import { EMPTY_COLLECTION } from './stream.constants';
import type { CollectionState, Collections, Listener, StreamStatus } from './stream.type';

class ScopeStream {
  private readonly scope: string;
  private readonly link: TabLink;
  private collections: Collections = new Map();
  private readonly listeners = new Map<string, Set<Listener>>();
  private readonly statusListeners = new Set<Listener>();
  private status: StreamStatus = 'idle';

  constructor(scope: string, link: TabLink) {
    this.scope = scope;
    this.link = link;
    link.route(scope, (message) => this.receive(message));
  }

  subscribe = (coll: string, listener: Listener): (() => void) => {
    const set = this.listeners.get(coll) ?? new Set<Listener>();
    set.add(listener);
    this.listeners.set(coll, set);
    if (this.status === 'idle') this.setStatus('connecting');
    this.link.want(this.scope, coll);
    return () => this.leave(coll, set, listener);
  };

  subscribeStatus = (listener: Listener): (() => void) => {
    this.statusListeners.add(listener);
    return () => {
      this.statusListeners.delete(listener);
    };
  };

  getCollection(coll: string): CollectionState {
    return this.collections.get(coll) ?? EMPTY_COLLECTION;
  }

  getStatus(): StreamStatus {
    return this.status;
  }

  private leave(coll: string, set: Set<Listener>, listener: Listener): void {
    set.delete(listener);
    if (set.size > 0 || this.listeners.get(coll) !== set) return;
    this.listeners.delete(coll);
    this.link.unwant(this.scope, coll);
    if (this.listeners.size === 0) this.setStatus('idle');
  }

  private receive(message: ScopeMessage): void {
    switch (message.type) {
      case 'snapshot':
        return this.onSnapshot(message);
      case 'changes':
        return this.onChanges(message);
      case 'reset':
        return this.onReset();
      case 'status':
        return this.onStatus(message.status);
    }
  }

  private onSnapshot(event: Extract<ScopeMessage, { type: 'snapshot' }>): void {
    this.collections = applySnapshot(this.collections, event);
    this.notify(event.coll);
  }

  private onChanges(event: Extract<ScopeMessage, { type: 'changes' }>): void {
    const applied = applyChanges(this.collections, event);
    this.collections = applied.collections;
    for (const coll of applied.touched) this.notify(coll);
  }

  private onReset(): void {
    const known = [...this.collections.keys()];
    this.collections = new Map();
    for (const coll of known) this.notify(coll);
  }

  private onStatus(status: StreamStatus): void {
    if (this.listeners.size > 0) this.setStatus(status);
  }

  private notify(coll: string): void {
    for (const listener of this.listeners.get(coll) ?? []) listener();
  }

  private setStatus(status: StreamStatus): void {
    if (status === this.status) return;
    this.status = status;
    for (const listener of this.statusListeners) listener();
  }
}

export { ScopeStream };
