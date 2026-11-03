/* @layer renderer-app @kind test */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { WorkerHost } from '../../src/data/hub/worker-host';
import type { HubMessage } from '../../src/data/hub/hub.type';
import { createFakeHubs } from '../helpers/fake-hub';
import { FakeLocks } from '../helpers/fake-locks';
import { FakeNetwork } from '../helpers/fake-network';
import { ANNOUNCE } from '../helpers/leader-rig';

const BEAT = { type: 'beat' };
const BYE = { type: 'bye' };
const RESET: HubMessage = { type: 'reset', scope: 'rotp' };

const createRig = (withLocks: boolean) => {
  const locks = new FakeLocks();
  const network = new FakeNetwork();
  const hubs = createFakeHubs();
  const host = new WorkerHost({ locks: withLocks ? locks : null, createHub: hubs.create, now: () => Date.now() });
  const settle = async (): Promise<void> => {
    for (let round = 0; round < 12; round += 1) {
      await Promise.resolve();
      network.flush();
    }
  };
  const tab = (tabId: string) => {
    const name = `port-${tabId}`;
    host.connect(network.open(name));
    const wire = network.open(name);
    const seen: unknown[] = [];
    wire.listen((data) => seen.push(data));
    const send = (msg: object, epoch = 0) => wire.post({ tab: tabId, epoch, msg });
    return { send, seen };
  };
  const hold = (tabId: string): void => {
    locks.request(`argus-tab-${tabId}`, {}, () => new Promise(() => undefined)).catch(() => undefined);
  };
  const join = async (tabId: string) => {
    hold(tabId);
    const joined = tab(tabId);
    joined.send(BEAT);
    await settle();
    return joined;
  };
  return { locks, network, hubs, host, settle, tab, join };
};

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(5000);
});

afterEach(() => {
  vi.useRealTimers();
});

describe('C4 worker host', { tags: ['critical'] }, () => {
  it('says hello to a tab on its first message, runs one hub for all tabs and routes both ways', async () => {
    const rig = createRig(true);
    const a = await rig.join('a');
    const b = await rig.join('b');
    expect(rig.hubs.hubs).toHaveLength(1);
    expect(a.seen).toEqual([{ tab: 'a', epoch: 5000, msg: { type: 'hub-hello', epoch: 5000, v: 1 } }]);
    expect(b.seen).toHaveLength(1);
    a.send(ANNOUNCE, 5000);
    await rig.settle();
    expect(rig.hubs.last().received).toContainEqual(['a', ANNOUNCE]);
    rig.hubs.last().port.post('b', RESET);
    rig.hubs.last().port.post('nobody', RESET);
    await rig.settle();
    expect(b.seen.at(-1)).toEqual({ tab: 'b', epoch: 5000, msg: RESET });
    expect(a.seen).toHaveLength(1);
  });

  it('ignores what is not a tab envelope', async () => {
    const rig = createRig(true);
    const a = rig.tab('a');
    a.send({ type: 'hub-hello', epoch: 1, v: 1 });
    a.send({ type: 'whatever' });
    await rig.settle();
    expect(rig.hubs.hubs).toHaveLength(0);
    expect(a.seen).toEqual([]);
  });

  it('drops a tab when its lock is granted to the worker, which means the tab is gone', async () => {
    const rig = createRig(true);
    await rig.join('a');
    await rig.join('b');
    expect(rig.locks.queued('argus-tab-a')).toBe(1);
    rig.locks.crash('argus-tab-a');
    await rig.settle();
    expect(rig.hubs.last().dropTab).toHaveBeenCalledTimes(1);
    expect(rig.hubs.last().dropTab).toHaveBeenCalledWith('a');
    expect(rig.locks.queued('argus-tab-b')).toBe(1);
  });

  it('forwards bye to the hub and stops watching the lock', async () => {
    const rig = createRig(true);
    const a = await rig.join('a');
    a.send(BYE);
    await rig.settle();
    expect(rig.hubs.last().received).toContainEqual(['a', BYE]);
    expect(rig.locks.queued('argus-tab-a')).toBe(0);
  });
});

describe('C4 worker host without Web Locks', { tags: ['critical'] }, () => {
  it('keeps a tab that beats and drops one that falls silent for thirty seconds', async () => {
    const rig = createRig(false);
    const live = rig.tab('live');
    const quiet = rig.tab('quiet');
    live.send(BEAT);
    quiet.send(BEAT);
    await rig.settle();
    for (let step = 0; step < 4; step += 1) {
      await vi.advanceTimersByTimeAsync(10_000);
      live.send(BEAT);
      await rig.settle();
    }
    expect(rig.hubs.last().dropTab).toHaveBeenCalledTimes(1);
    expect(rig.hubs.last().dropTab).toHaveBeenCalledWith('quiet');
  });
});

describe('C4 worker host lifetime', { tags: ['critical'] }, () => {
  it('disposes the hub a while after the last tab leaves, and makes a new one for the next tab', async () => {
    const rig = createRig(true);
    const a = await rig.join('a');
    a.send(BYE);
    await rig.settle();
    expect(rig.hubs.last().dispose).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(15_000);
    expect(rig.hubs.hubs[0]?.dispose).toHaveBeenCalledTimes(1);
    const b = await rig.join('b');
    expect(rig.hubs.hubs).toHaveLength(2);
    expect(b.seen).toHaveLength(1);
  });

  it('keeps the hub when a tab comes back inside the idle window, as a reload does', async () => {
    const rig = createRig(true);
    const a = await rig.join('a');
    a.send(BYE);
    await rig.settle();
    await vi.advanceTimersByTimeAsync(5000);
    await rig.join('a2');
    await vi.advanceTimersByTimeAsync(60_000);
    expect(rig.hubs.hubs).toHaveLength(1);
    expect(rig.hubs.last().dispose).not.toHaveBeenCalled();
  });
});
