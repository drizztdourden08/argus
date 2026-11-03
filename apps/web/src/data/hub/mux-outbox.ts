/* @layer renderer-app @kind logic */
import { MAX_BATCH } from './hub.constants';
import type { MuxOp, MuxPostResult, OutboxHandlers } from './hub.type';

class MuxOutbox {
  private readonly handlers: OutboxHandlers;
  private conn: string | null = null;
  private queue: MuxOp[] = [];
  private sending = false;
  private flushQueued = false;

  constructor(handlers: OutboxHandlers) {
    this.handlers = handlers;
  }

  attach(conn: string, ops: readonly MuxOp[]): void {
    this.conn = conn;
    this.queue = [...ops];
    this.sending = false;
    this.schedule();
  }

  detach(): void {
    this.conn = null;
    this.queue = [];
    this.sending = false;
  }

  push(ops: readonly MuxOp[]): void {
    if (this.conn === null) return;
    this.queue.push(...ops);
    this.schedule();
  }

  private schedule(): void {
    if (this.flushQueued) return;
    this.flushQueued = true;
    queueMicrotask(() => {
      this.flushQueued = false;
      void this.pump();
    });
  }

  private async pump(): Promise<void> {
    const conn = this.conn;
    if (conn === null || this.sending || this.queue.length === 0) return;
    this.sending = true;
    const ops = this.queue.splice(0, MAX_BATCH);
    const result = await this.handlers.post(conn, ops).catch(() => null);
    if (conn !== this.conn) return;
    this.sending = false;
    if (!result) return this.handlers.onFailed();
    if (result.status === 404) return this.handlers.onGone();
    if (result.status !== 200) return this.handlers.onFailed();
    this.settle(ops, result);
  }

  private settle(ops: readonly MuxOp[], result: MuxPostResult): void {
    this.handlers.onDelivered();
    result.results.forEach((outcome, index) => {
      const op = ops[index];
      if (op && 'error' in outcome) this.handlers.onOpError(op, outcome.error);
    });
    void this.pump();
  }
}

export { MuxOutbox };
