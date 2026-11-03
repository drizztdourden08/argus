/* @layer renderer-app @kind logic */
import { envelope } from './envelope';
import { envelopeGuard } from './envelope-guard';
import { BEAT_MS, HUB_TYPES } from './hub-transport.constants';
import type { WorkerTabDeps, Wire } from './hub-transport.type';
import type { HubMessage, TabMessage } from './hub.type';
import { ListenerTab } from './listener-tab';
import { TabGate } from './tab-gate';
import { holdTabLock } from './tab-lock';

const isHubEnvelope = envelopeGuard<HubMessage>(HUB_TYPES);

class WorkerTab extends ListenerTab {
  private readonly wire: Wire;
  private readonly gate = new TabGate((message) => this.post(message));
  private readonly releaseTabLock: () => void;
  private readonly beat: ReturnType<typeof setInterval>;
  private epoch = 0;

  constructor(deps: WorkerTabDeps) {
    super(deps.tabId);
    this.wire = deps.wire;
    this.releaseTabLock = holdTabLock(deps.locks, deps.tabId, () => this.gate.open());
    this.wire.listen((data) => this.receive(data));
    this.beat = setInterval(() => this.send({ type: 'beat' }), BEAT_MS);
    this.send({ type: 'beat' });
  }

  send(message: TabMessage): void {
    if (!this.closed) this.gate.pass(message);
  }

  protected teardown(): void {
    clearInterval(this.beat);
    this.releaseTabLock();
    this.wire.close();
  }

  private post(message: TabMessage): void {
    if (!this.closed) this.wire.post(envelope(this.tabId, this.epoch, message));
  }

  private receive(data: unknown): void {
    if (!isHubEnvelope(data) || data.tab !== this.tabId) return;
    if (data.msg.type === 'hub-hello') this.epoch = data.epoch;
    this.listeners.emit(data.msg);
  }
}

export { WorkerTab };
