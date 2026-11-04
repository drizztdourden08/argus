/* @layer renderer-app @kind logic */
import type { TierWatchDeps } from './fallback-tab.type';
import type { HubMessage } from './hub.type';

const reachable = (status: number | null): boolean => status !== null && status >= 200 && status < 300;

class TierWatch {
  private readonly deps: TierWatchDeps;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private judging = false;
  private done = false;

  constructor(deps: TierWatchDeps) {
    this.deps = deps;
  }

  see(message: HubMessage): void {
    if (this.done || message.type !== 'status') return;
    if (message.status === 'live' || message.status === 'unauthorized') return this.stop();
    if (message.status === 'unavailable') {
      void this.judge();
      return;
    }
    this.timer ??= setTimeout(() => void this.judge(), this.deps.waitMs);
  }

  stop(): void {
    this.done = true;
    this.clearTimer();
  }

  private async judge(): Promise<void> {
    this.clearTimer();
    if (this.judging) return;
    this.judging = true;
    const status = await this.deps.health().catch(() => null);
    this.judging = false;
    if (this.done) return;
    if (!reachable(status)) {
      this.timer = setTimeout(() => void this.judge(), this.deps.waitMs);
      return;
    }
    this.stop();
    this.deps.onFail();
  }

  private clearTimer(): void {
    if (this.timer !== null) clearTimeout(this.timer);
    this.timer = null;
  }
}

export { TierWatch };
