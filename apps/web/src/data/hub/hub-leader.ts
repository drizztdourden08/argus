/* @layer renderer-app @kind logic */
import { envelope } from './envelope';
import { envelopeGuard } from './envelope-guard';
import { BUS_NAME, EVERY_TAB, HUB_TYPES, INBOX_PREFIX, LEADER_LOCK } from './hub-transport.constants';
import type { LeaderDeps, Wire } from './hub-transport.type';
import type { HubMessage, TabMessage } from './hub.type';
import { LeaderHost } from './leader-host';
import { ListenerTab } from './listener-tab';
import { holdTabLock } from './tab-lock';

const isHubEnvelope = envelopeGuard<HubMessage>(HUB_TYPES);

class LeaderTab extends ListenerTab {
  private readonly deps: LeaderDeps;
  private readonly bus: Wire;
  private readonly inbox: Wire;
  private readonly election = new AbortController();
  private readonly releaseTabLock: () => void;
  private epoch = 0;
  private host: LeaderHost | null = null;
  private stepDown: (() => void) | null = null;

  constructor(deps: LeaderDeps) {
    super(deps.tabId);
    this.deps = deps;
    this.bus = deps.openWire(BUS_NAME);
    this.inbox = deps.openWire(`${INBOX_PREFIX}${deps.tabId}`);
    this.releaseTabLock = holdTabLock(deps.locks, deps.tabId);
    this.inbox.listen((data) => this.fromHub(data));
    this.bus.listen((data) => this.fromBus(data));
    deps.locks.request(LEADER_LOCK, { signal: this.election.signal }, () => this.lead()).catch(() => undefined);
    this.send({ type: 'beat' });
  }

  send(message: TabMessage): void {
    if (this.host) return this.host.receiveOwn(message);
    this.bus.post(envelope(this.tabId, this.epoch, message));
  }

  protected teardown(): void {
    this.election.abort();
    this.host?.stop();
    this.host = null;
    this.stepDown?.();
    this.releaseTabLock();
    this.inbox.close();
    this.bus.close();
  }

  private lead(): Promise<void> {
    if (this.closed) return Promise.resolve();
    return new Promise<void>((resolve) => {
      this.stepDown = resolve;
      this.epoch = Math.max(this.deps.now(), this.epoch + 1);
      this.host = new LeaderHost({
        deps: this.deps,
        epoch: this.epoch,
        bus: this.bus,
        deliver: (message) => this.listeners.emit(message),
      });
      this.host.start();
    });
  }

  private fromBus(data: unknown): void {
    if (this.host) return this.host.receive(data);
    if (isHubEnvelope(data) && data.tab === EVERY_TAB) this.fromHub(data);
  }

  private fromHub(data: unknown): void {
    if (!isHubEnvelope(data)) return;
    const { epoch, msg } = data;
    if (msg.type === 'hub-hello') return this.hello(epoch, msg);
    if (epoch === this.epoch) this.listeners.emit(msg);
  }

  private hello(epoch: number, message: HubMessage): void {
    if (epoch <= this.epoch) return;
    this.epoch = epoch;
    this.listeners.emit(message);
  }
}

export { LeaderTab };
