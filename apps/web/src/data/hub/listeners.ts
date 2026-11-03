/* @layer renderer-app @kind logic */
import type { HubMessage } from './hub.type';
import type { MessageListener } from './tab-handle.type';

class Listeners {
  private readonly all = new Set<MessageListener>();

  add(listener: MessageListener): () => void {
    this.all.add(listener);
    return () => {
      this.all.delete(listener);
    };
  }

  emit(message: HubMessage): void {
    for (const listener of [...this.all]) listener(message);
  }

  clear(): void {
    this.all.clear();
  }
}

export { Listeners };
