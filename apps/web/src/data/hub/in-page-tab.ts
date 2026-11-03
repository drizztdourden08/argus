/* @layer renderer-app @kind logic */
import { helloOf } from './hub-hello';
import type { HubLike, InPageDeps } from './hub-transport.type';
import type { TabMessage } from './hub.type';
import { Listeners } from './listeners';
import type { MessageListener, TabHandle } from './tab-handle.type';

class InPageTab implements TabHandle {
  readonly tabId: string;
  private readonly hub: HubLike;
  private readonly listeners = new Listeners();
  private closed = false;

  constructor(deps: InPageDeps) {
    this.tabId = deps.tabId;
    this.hub = deps.createHub({ post: (_tabId, message) => this.listeners.emit(message) });
    queueMicrotask(() => {
      if (!this.closed) this.listeners.emit(helloOf(deps.now()));
    });
  }

  send(message: TabMessage): void {
    if (!this.closed) this.hub.receive(this.tabId, message);
  }

  onMessage(listener: MessageListener): () => void {
    return this.listeners.add(listener);
  }

  close(): void {
    if (this.closed) return;
    this.closed = true;
    this.hub.dispose();
    this.listeners.clear();
  }
}

export { InPageTab };
