/* @layer renderer-app @kind logic */
import type { FallbackOptions, TabTier } from './fallback-tab.type';
import type { HubMessage, TabMessage } from './hub.type';
import { Listeners } from './listeners';
import type { MessageListener, TabHandle } from './tab-handle.type';
import { TierWatch } from './tier-watch';

const tryOpen = (tier: TabTier | undefined): TabHandle | null => {
  if (!tier) return null;
  try {
    return tier();
  } catch {
    return null;
  }
};

class FallbackTab implements TabHandle {
  readonly tabId: string;
  private readonly options: FallbackOptions;
  private readonly listeners = new Listeners();
  private current: TabHandle | null = null;
  private unlisten: () => void = () => undefined;
  private watch: TierWatch | null = null;
  private next = 0;
  private onLast = false;
  private closed = false;

  constructor(options: FallbackOptions) {
    this.tabId = options.tabId;
    this.options = options;
    this.adopt();
  }

  send(message: TabMessage): void {
    if (!this.closed) this.current?.send(message);
  }

  onMessage(listener: MessageListener): () => void {
    return this.listeners.add(listener);
  }

  close(): void {
    if (this.closed) return;
    this.closed = true;
    this.drop();
    this.listeners.clear();
  }

  private adopt(): void {
    const handle = this.openNext();
    this.current = handle;
    this.unlisten = handle.onMessage((message) => this.deliver(message));
    const { health, waitMs } = this.options;
    this.watch = this.onLast || !health ? null : new TierWatch({ health, waitMs, onFail: () => this.fallBack() });
  }

  private openNext(): TabHandle {
    const { tiers, last } = this.options;
    while (this.next < tiers.length) {
      const handle = tryOpen(tiers[this.next]);
      this.next += 1;
      if (handle) return handle;
    }
    this.onLast = true;
    return last();
  }

  private deliver(message: HubMessage): void {
    this.watch?.see(message);
    this.listeners.emit(message);
  }

  private fallBack(): void {
    if (this.closed) return;
    this.drop();
    this.adopt();
  }

  private drop(): void {
    this.watch?.stop();
    this.watch = null;
    this.unlisten();
    this.current?.close();
    this.current = null;
  }
}

export { FallbackTab };
