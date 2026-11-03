/* @layer renderer-app @kind logic */
import type { TabMessage } from './hub.type';

class TabGate {
  private readonly post: (message: TabMessage) => void;
  private waiting: TabMessage[] | null = [];

  constructor(post: (message: TabMessage) => void) {
    this.post = post;
  }

  pass(message: TabMessage): void {
    if (this.waiting) this.waiting.push(message);
    else this.post(message);
  }

  open(): void {
    const waiting = this.waiting ?? [];
    this.waiting = null;
    for (const message of waiting) this.post(message);
  }
}

export { TabGate };
