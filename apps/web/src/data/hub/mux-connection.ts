/* @layer renderer-app @kind logic */
import type { SourceLike, StreamStatus } from '../stream/stream.type';
import { FAILURES_BEFORE_UNAVAILABLE, GONE_RETRIES_AT_ONCE, MUX_EVENTS, MUX_URL } from './hub.constants';
import type { MuxEventName, MuxFrame, MuxHello, MuxOp, MuxOptions } from './hub.type';
import { MuxOutbox } from './mux-outbox';
import { reconnectDelay } from './reconnect-delay';

class MuxConnection {
  private readonly options: MuxOptions;
  private readonly outbox: MuxOutbox;
  private status: StreamStatus = 'idle';
  private source: SourceLike | null = null;
  private wanted = false;
  private failures = 0;
  private gone = 0;
  private timer: ReturnType<typeof setTimeout> | null = null;

  constructor(options: MuxOptions) {
    this.options = options;
    this.outbox = new MuxOutbox({
      post: options.post,
      onDelivered: () => {
        this.gone = 0;
      },
      onGone: () => this.onGone(),
      onFailed: () => this.fail(),
      onOpError: options.onOpError,
    });
  }

  open(): void {
    if (this.wanted) return;
    this.wanted = true;
    this.connect();
  }

  close(): void {
    this.wanted = false;
    this.clearTimer();
    this.closeSource();
    this.failures = 0;
    this.gone = 0;
    this.setStatus('idle');
  }

  send(ops: readonly MuxOp[]): void {
    this.outbox.push(ops);
  }

  getStatus(): StreamStatus {
    return this.status;
  }

  private connect(): void {
    this.clearTimer();
    this.closeSource();
    const source = this.options.openSource(MUX_URL);
    this.source = source;
    for (const name of MUX_EVENTS) source.addEventListener(name, (event) => this.onEvent(source, name, event.data));
    source.addEventListener('error', () => this.onError(source));
    if (this.status === 'idle') this.setStatus('connecting');
  }

  private onEvent(source: SourceLike, name: MuxEventName, data: string): void {
    if (source !== this.source) return;
    this.failures = 0;
    this.setStatus('live');
    if (name === 'hello') this.outbox.attach((JSON.parse(data) as MuxHello).conn, this.options.resume());
    else this.options.onFrame({ event: name, data: JSON.parse(data) } as MuxFrame);
  }

  private onError(source: SourceLike): void {
    if (source === this.source) this.fail();
  }

  private fail(): void {
    this.closeSource();
    this.failures += 1;
    if (this.failures === 1) void this.checkAuth();
    if (this.failures >= FAILURES_BEFORE_UNAVAILABLE) {
      if (this.status !== 'unauthorized') this.setStatus('unavailable');
    } else if (this.status === 'live') this.setStatus('connecting');
    this.retryIn(reconnectDelay(this.failures));
  }

  private onGone(): void {
    this.closeSource();
    this.gone += 1;
    if (this.status === 'live') this.setStatus('connecting');
    if (this.gone <= GONE_RETRIES_AT_ONCE) return this.connect();
    this.retryIn(reconnectDelay(this.gone - GONE_RETRIES_AT_ONCE));
  }

  private retryIn(delay: number): void {
    this.clearTimer();
    this.timer = setTimeout(() => {
      this.timer = null;
      if (this.wanted) this.connect();
    }, delay);
  }

  private async checkAuth(): Promise<void> {
    const status = await this.options.health().catch(() => null);
    if (status === 401 || status === 403) this.setStatus('unauthorized');
  }

  private setStatus(status: StreamStatus): void {
    if (status === this.status) return;
    this.status = status;
    this.options.onStatus(status);
  }

  private closeSource(): void {
    this.source?.close();
    this.source = null;
    this.outbox.detach();
  }

  private clearTimer(): void {
    if (this.timer !== null) clearTimeout(this.timer);
    this.timer = null;
  }
}

export { MuxConnection };
