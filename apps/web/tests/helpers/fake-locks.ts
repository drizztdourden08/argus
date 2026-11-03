/* @layer renderer-app @kind test */
import type { LockManagerLike } from '../../src/data/hub/hub-transport.type';

interface Waiter {
  run: () => void;
}

class FakeLocks implements LockManagerLike {
  private readonly holders = new Map<string, () => void>();
  private readonly queues = new Map<string, Waiter[]>();

  request = (name: string, options: Parameters<LockManagerLike['request']>[1], callback: () => unknown): Promise<unknown> =>
    new Promise((resolve, reject) => {
      const waiter: Waiter = { run: () => this.grant(name, callback, resolve) };
      this.queues.set(name, [...(this.queues.get(name) ?? []), waiter]);
      options.signal?.addEventListener('abort', () => {
        this.queues.set(name, (this.queues.get(name) ?? []).filter((queued) => queued !== waiter));
        reject(new DOMException('aborted', 'AbortError'));
      });
      if (!this.holders.has(name)) this.advance(name);
    });

  crash(name: string): void {
    this.holders.get(name)?.();
  }

  isHeld(name: string): boolean {
    return this.holders.has(name);
  }

  queued(name: string): number {
    return (this.queues.get(name) ?? []).length;
  }

  private grant(name: string, callback: () => unknown, resolve: (value: unknown) => void): void {
    let force: () => void = () => undefined;
    const forced = new Promise<void>((done) => {
      force = done;
    });
    this.holders.set(name, force);
    Promise.race([Promise.resolve().then(callback), forced])
      .then(() => {
        this.holders.delete(name);
        resolve(undefined);
        this.advance(name);
      })
      .catch(() => undefined);
  }

  private advance(name: string): void {
    const [next, ...rest] = this.queues.get(name) ?? [];
    this.queues.set(name, rest);
    next?.run();
  }
}

export { FakeLocks };
