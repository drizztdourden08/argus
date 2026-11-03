/* @layer renderer-app @kind logic */
import type { TabMessage } from './hub.type';
import { Listeners } from './listeners';
import type { MessageListener, TabHandle } from './tab-handle.type';

abstract class ListenerTab implements TabHandle {
  readonly tabId: string;
  protected readonly listeners = new Listeners();
  protected closed = false;

  protected constructor(tabId: string) {
    this.tabId = tabId;
  }

  abstract send(message: TabMessage): void;

  onMessage(listener: MessageListener): () => void {
    return this.listeners.add(listener);
  }

  close(): void {
    if (this.closed) return;
    this.send({ type: 'bye' });
    this.closed = true;
    this.teardown();
    this.listeners.clear();
  }

  protected abstract teardown(): void;
}

export { ListenerTab };
