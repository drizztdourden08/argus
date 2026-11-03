/* @layer renderer-app @kind test */
import { TabLink } from '../../src/data/hub/tab-link';
import type { HubMessage, TabHandle, TabMessage } from '../../src/data/hub';

type Deliver = (message: HubMessage) => void;

const FRAMES: Record<string, (scope: string, body: object) => HubMessage | null> = {
  snapshot: (scope, body) => ({ ...(body as Extract<HubMessage, { type: 'snapshot' }>), type: 'snapshot', scope }),
  changes: (scope, body) => ({ ...(body as Extract<HubMessage, { type: 'changes' }>), type: 'changes', scope }),
  reset: (scope) => ({ type: 'reset', scope }),
  ready: (scope) => ({ type: 'status', scope, status: 'live' }),
  status: (scope, body) => ({ ...(body as Extract<HubMessage, { type: 'status' }>), type: 'status', scope }),
};

const LIVE_ON = new Set(['snapshot', 'changes']);

class FakeSource {
  readonly scope: string;
  readonly colls = new Set<string>();
  closed = false;
  private readonly deliver: Deliver;
  private live = false;

  constructor(scope: string, deliver: Deliver) {
    this.scope = scope;
    this.deliver = deliver;
  }

  get url(): string {
    return `/api/p/${this.scope}/stream?c=${[...this.colls].join(',')}`;
  }

  emit(type: string, body: object = {}): void {
    const frame = FRAMES[type]?.(this.scope, body);
    if (!frame) return;
    this.deliver(frame);
    if (LIVE_ON.has(type) && !this.live) this.deliver({ type: 'status', scope: this.scope, status: 'live' });
    if (LIVE_ON.has(type) || type === 'ready') this.live = true;
  }
}

class FakeHandle implements TabHandle {
  tabId = 'tab-test';
  readonly sent: TabMessage[] = [];
  readonly sources: FakeSource[] = [];
  private readonly listeners = new Set<Deliver>();

  send(message: TabMessage): void {
    this.sent.push(message);
    if (message.type === 'want') this.want(message.scope, message.coll);
    else if (message.type === 'unwant') this.unwant(message.scope, message.coll);
  }

  onMessage(listener: Deliver): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  close(): void {
    this.listeners.clear();
  }

  deliver: Deliver = (message) => {
    for (const listener of this.listeners) listener(message);
  };

  private openOf(scope: string): FakeSource | undefined {
    return this.sources.find((source) => source.scope === scope && !source.closed);
  }

  private want(scope: string, coll: string): void {
    let source = this.openOf(scope);
    if (!source) {
      source = new FakeSource(scope, this.deliver);
      this.sources.push(source);
    }
    source.colls.add(coll);
  }

  private unwant(scope: string, coll: string): void {
    const source = this.openOf(scope);
    source?.colls.delete(coll);
    if (source?.colls.size === 0) source.closed = true;
  }
}

interface FakeSources {
  open: TabLink;
  handle: FakeHandle;
  sources: FakeSource[];
  sent: TabMessage[];
}

const createFakeSources = (): FakeSources => {
  const handle = new FakeHandle();
  return { open: new TabLink(handle), handle, sources: handle.sources, sent: handle.sent };
};

export { createFakeSources, FakeSource };
