/* @layer renderer-app @kind logic */
import { applyChanges } from './apply-changes';
import { applySnapshot } from './apply-snapshot';
import { reconnectDelay } from './reconnect-delay';
import { EMPTY_COLLECTION, FAILURES_BEFORE_UNAVAILABLE, SOURCE_EVENTS } from './stream.constants';
import { streamUrl } from './stream-url';
import type { ChangesEvent, CollectionState, Listener, Collections, OpenSource, SnapshotEvent, SourceLike, StreamStatus } from './stream.type';

class ScopeStream {
  private readonly scope: string;
  private readonly openSource: OpenSource;
  private collections: Collections = new Map();
  private readonly listeners = new Map<string, Set<Listener>>();
  private readonly statusListeners = new Set<Listener>();
  private status: StreamStatus = 'idle';
  private source: SourceLike | null = null;
  private opened: ReadonlySet<string> = new Set();
  private seq = 0;
  private failures = 0;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private syncQueued = false;

  constructor(scope: string, openSource: OpenSource) {
    this.scope = scope;
    this.openSource = openSource;
  }

  subscribe = (coll: string, listener: Listener): (() => void) => {
    const set = this.listeners.get(coll) ?? new Set<Listener>();
    set.add(listener);
    this.listeners.set(coll, set);
    this.queueSync();
    return () => {
      set.delete(listener);
      if (set.size === 0 && this.listeners.get(coll) === set) this.listeners.delete(coll);
      this.queueSync();
    };
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

  private queueSync(): void {
    if (this.syncQueued) return;
    this.syncQueued = true;
    queueMicrotask(() => {
      this.syncQueued = false;
      this.sync();
    });
  }

  private sync(): void {
    const wanted = [...this.listeners.keys()];
    if (wanted.length === 0) this.stop();
    else if (!this.source || !wanted.every((coll) => this.opened.has(coll))) this.connect(false);
  }

  private stop(): void {
    this.clearTimer();
    this.closeSource();
    this.failures = 0;
    this.setStatus('idle');
  }

  private connect(resume: boolean): void {
    this.clearTimer();
    this.closeSource();
    const wanted = [...this.listeners.keys()];
    const caughtUp = resume && this.seq > 0 && wanted.every((coll) => this.collections.get(coll)?.ready);
    this.opened = new Set(wanted);
    const source = this.openSource(streamUrl(this.scope, wanted, caughtUp ? this.seq : null));
    this.source = source;
    this.attach(source);
    if (this.status !== 'live') this.setStatus('connecting');
  }

  private attach(source: SourceLike): void {
    for (const name of SOURCE_EVENTS) source.addEventListener(name, (event) => this.onEvent(source, name, event.data));
    source.addEventListener('error', () => this.onError(source));
  }

  private onEvent(source: SourceLike, name: (typeof SOURCE_EVENTS)[number], data: string): void {
    if (source !== this.source) return;
    if (name === 'reset') return this.onReset();
    const body = JSON.parse(data) as { seq: number };
    if (name === 'snapshot') this.onSnapshot(body as SnapshotEvent);
    else if (name === 'changes') this.onChanges(body as ChangesEvent);
    this.seq = Math.max(this.seq, body.seq);
    this.markLive();
  }

  private onSnapshot(event: SnapshotEvent): void {
    this.collections = applySnapshot(this.collections, event);
    this.notify(event.coll);
  }

  private onChanges(event: ChangesEvent): void {
    const applied = applyChanges(this.collections, event);
    this.collections = applied.collections;
    for (const coll of applied.touched) this.notify(coll);
  }

  private onReset(): void {
    const known = [...this.collections.keys()];
    this.collections = new Map();
    this.seq = 0;
    for (const coll of known) this.notify(coll);
    this.connect(false);
  }

  private onError(source: SourceLike): void {
    if (source !== this.source) return;
    this.closeSource();
    this.failures += 1;
    if (this.failures === 1) void this.checkAuth();
    if (this.failures >= FAILURES_BEFORE_UNAVAILABLE) {
      if (this.status !== 'unauthorized') this.setStatus('unavailable');
    } else if (this.status === 'live') this.setStatus('connecting');
    this.timer = setTimeout(() => this.reconnect(), reconnectDelay(this.failures));
  }

  private reconnect(): void {
    this.timer = null;
    if (this.listeners.size > 0) this.connect(true);
  }

  private async checkAuth(): Promise<void> {
    const response = await fetch('/api/health').catch(() => null);
    if (response && (response.status === 401 || response.status === 403)) this.setStatus('unauthorized');
  }

  private markLive(): void {
    this.failures = 0;
    this.setStatus('live');
  }

  private notify(coll: string): void {
    for (const listener of this.listeners.get(coll) ?? []) listener();
  }

  private setStatus(status: StreamStatus): void {
    if (status === this.status) return;
    this.status = status;
    for (const listener of this.statusListeners) listener();
  }

  private closeSource(): void {
    this.source?.close();
    this.source = null;
  }

  private clearTimer(): void {
    if (this.timer !== null) clearTimeout(this.timer);
    this.timer = null;
  }
}

export { ScopeStream };
