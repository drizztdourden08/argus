/* @layer renderer-app @kind test */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { connectHub } from '../../src/data/hub';
import type { HubMessage } from '../../src/data/hub';
import type { ConnectEnv } from '../../src/data/hub/hub-transport.type';
import { TabLink } from '../../src/data/hub/tab-link';
import { createFakeHubs } from '../helpers/fake-hub';
import { FakeLocks } from '../helpers/fake-locks';
import { FakeNetwork, settleNetwork } from '../helpers/fake-network';

const BYE = { type: 'bye' };
const fromWorker = (msg: HubMessage) => ({ tab: 't1', epoch: 9, msg });
const status = (value: 'connecting' | 'live' | 'unavailable'): HubMessage => ({ type: 'status', scope: 'argus', status: value });

const createRig = (reach: number | null, withWorker = true) => {
  const locks = new FakeLocks();
  const network = new FakeNetwork();
  const hubs = createFakeHubs();
  const health = vi.fn(() => Promise.resolve(reach));
  const tabSide = network.open('worker-port');
  const workerSide = network.open('worker-port');
  const seen: unknown[] = [];
  workerSide.listen((data) => seen.push(data));
  const env: ConnectEnv = {
    tabId: 't1',
    locks,
    openWorker: withWorker ? () => tabSide : null,
    openWire: network.open,
    createHub: hubs.create,
    now: () => 77,
    onHide: vi.fn(),
    health,
  };
  const settle = (): Promise<void> => settleNetwork(network);
  return { locks, hubs, health, workerSide, seen, env, settle };
};

const hellos = (got: HubMessage[]): HubMessage[] => got.filter((message) => message.type === 'hub-hello');

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('C4 hub tier fallback from the worker', { tags: ['critical'] }, () => {
  it('leaves a worker that never goes live for the leader tier and replays every subscription', async () => {
    const rig = createRig(200);
    const handle = connectHub(rig.env);
    const link = new TabLink(handle);
    const got: HubMessage[] = [];
    handle.onMessage((message) => got.push(message));
    await rig.settle();
    rig.workerSide.post(fromWorker({ type: 'hub-hello', epoch: 9, v: 1 }));
    await rig.settle();
    link.want('argus', 'threads');
    rig.workerSide.post(fromWorker(status('connecting')));
    await rig.settle();
    await vi.advanceTimersByTimeAsync(4_900);
    expect(rig.hubs.create).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(100);
    await rig.settle();
    expect(rig.health).toHaveBeenCalledTimes(1);
    expect(rig.seen.at(-1)).toEqual({ tab: 't1', epoch: 9, msg: BYE });
    expect(rig.locks.isHeld('argus-hub-leader')).toBe(true);
    expect(hellos(got)).toEqual([
      { type: 'hub-hello', epoch: 9, v: 1 },
      { type: 'hub-hello', epoch: 77, v: 1 },
    ]);
    const announce = rig.hubs.last().received.find(([, message]) => message.type === 'announce');
    expect(announce?.[1]).toMatchObject({ type: 'announce', tabId: 't1', wants: [{ scope: 'argus', coll: 'threads' }] });
  });

  it('checks at once when the worker says unavailable', async () => {
    const rig = createRig(200);
    connectHub(rig.env);
    await rig.settle();
    rig.workerSide.post(fromWorker(status('unavailable')));
    await rig.settle();
    expect(rig.health).toHaveBeenCalledTimes(1);
    expect(rig.locks.isHeld('argus-hub-leader')).toBe(true);
  });

  it('stays on the worker while the page cannot reach the server either, and stops watching once live', async () => {
    const rig = createRig(null);
    connectHub(rig.env);
    await rig.settle();
    rig.workerSide.post(fromWorker(status('connecting')));
    await rig.settle();
    await vi.advanceTimersByTimeAsync(10_000);
    expect(rig.health).toHaveBeenCalledTimes(2);
    rig.workerSide.post(fromWorker(status('live')));
    await rig.settle();
    await vi.advanceTimersByTimeAsync(20_000);
    rig.workerSide.post(fromWorker(status('unavailable')));
    await rig.settle();
    expect(rig.health).toHaveBeenCalledTimes(2);
    expect(rig.hubs.create).not.toHaveBeenCalled();
  });
});

describe('C4 hub tier fallback from the leader', { tags: ['critical'] }, () => {
  it('leaves a leader tier that never goes live for the in-tab hub, which is never watched', async () => {
    const rig = createRig(200, false);
    connectHub(rig.env);
    await rig.settle();
    expect(rig.locks.isHeld('argus-hub-leader')).toBe(true);
    rig.hubs.last().port.post('t1', status('connecting'));
    await vi.advanceTimersByTimeAsync(5_000);
    await rig.settle();
    expect(rig.hubs.create).toHaveBeenCalledTimes(2);
    expect(rig.hubs.hubs[0]?.dispose).toHaveBeenCalledTimes(1);
    expect(rig.locks.isHeld('argus-hub-leader')).toBe(false);
    rig.hubs.last().port.post('t1', status('connecting'));
    await vi.advanceTimersByTimeAsync(20_000);
    expect(rig.health).toHaveBeenCalledTimes(1);
  });
});
