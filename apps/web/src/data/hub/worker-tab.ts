/* @layer renderer-app @kind logic */
import { envelope } from './envelope';
import { envelopeGuard } from './envelope-guard';
import { BEAT_MS, HUB_TYPES } from './hub-transport.constants';
import type { WorkerTabDeps, Wire } from './hub-transport.type';
import type { HubMessage, TabMessage } from './hub.type';
import { ListenerTab } from './listener-tab';
import { holdTabLock } from './tab-lock';

const isHubEnvelope = envelopeGuard<HubMessage>(HUB_TYPES);

class WorkerTab extends ListenerTab {
  private readonly wire: Wire;
  private readonly releaseTabLock: () => void;
  private readonly beat: ReturnType<typeof setInterval>;
  private epoch = 0;

  constructor(deps: WorkerTabDeps) {
    super(deps.tabId);
    this.wire = deps.wire;
    this.releaseTabLock = deps.locks ? holdTabLock(deps.locks, deps.tabId) : () => undefined;
    this.wire.listen((data) => this.receive(data));
    this.beat = setInterval(() => this.send({ type: 'beat' }), BEAT_MS);
    this.send({ type: 'beat' });
  }

  send(message: TabMessage): void {
    if (!this.closed) this.wire.post(envelope(this.tabId, this.epoch, message));
  }

  protected teardown(): void {
    clearInterval(this.beat);
    this.releaseTabLock();
    this.wire.close();
  }

  private receive(data: unknown): void {
    if (!isHubEnvelope(data) || data.tab !== this.tabId) return;
    if (data.msg.type === 'hub-hello') this.epoch = data.epoch;
    this.listeners.emit(data.msg);
  }
}

export { WorkerTab };
