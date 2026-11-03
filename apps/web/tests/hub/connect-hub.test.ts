/* @layer renderer-app @kind test */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { connectHub } from '../../src/data/hub';
import type { HubMessage } from '../../src/data/hub';
import type { ConnectEnv } from '../../src/data/hub/hub-transport.type';
import { createFakeHubs } from '../helpers/fake-hub';
import { FakeLocks } from '../helpers/fake-locks';
import { FakeNetwork } from '../helpers/fake-network';
import { ANNOUNCE } from '../helpers/leader-rig';

const BEAT = { type: 'beat' };
const BYE = { type: 'bye' };

const createRig = (override: Partial<ConnectEnv> = {}) => {
  const locks = new FakeLocks();
  const network = new FakeNetwork();
  const hubs = createFakeHubs();
  const onHide = vi.fn<(listener: () => void) => void>();
  const env: ConnectEnv = {
    tabId: 't1',
    locks: null,
    openWorker: null,
    openWire: null,
    createHub: hubs.create,
    now: () => 77,
    onHide,
    ...override,
  };
  const settle = async (): Promise<void> => {
    for (let round = 0; round < 12; round += 1) {
      await Promise.resolve();
      network.flush();
    }
  };
  return { locks, network, hubs, onHide, env, settle };
};

const listen = (handle: ReturnType<typeof connectHub>): HubMessage[] => {
  const got: HubMessage[] = [];
  handle.onMessage((message) => got.push(message));
  return got;
};

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('C4 connectHub picker', { tags: ['critical'] }, () => {
  it('prefers the shared worker over the leader tier and holds the tab lock', async () => {
    const rig = createRig();
    const port = rig.network.open('worker-port');
    const worker = rig.network.open('worker-port');
    const seen: unknown[] = [];
    worker.listen((data) => seen.push(data));
    rig.env.locks = rig.locks;
    rig.env.openWire = rig.network.open;
    rig.env.openWorker = () => port;
    const handle = connectHub(rig.env);
    await rig.settle();
    expect(handle.tabId).toBe('t1');
    expect(seen).toEqual([{ tab: 't1', epoch: 0, msg: BEAT }]);
    expect(rig.locks.isHeld('argus-tab-t1')).toBe(true);
    expect(rig.locks.isHeld('argus-hub-leader')).toBe(false);
    expect(rig.hubs.create).not.toHaveBeenCalled();
  });

  it('falls to the leader tier when the worker cannot be made', async () => {
    const rig = createRig({
      openWorker: () => {
        throw new Error('no module workers');
      },
    });
    rig.env.locks = rig.locks;
    rig.env.openWire = rig.network.open;
    const handle = connectHub(rig.env);
    const got = listen(handle);
    await rig.settle();
    expect(rig.locks.isHeld('argus-hub-leader')).toBe(true);
    expect(rig.hubs.create).toHaveBeenCalledTimes(1);
    expect(got).toEqual([{ type: 'hub-hello', epoch: 77, v: 1 }]);
    handle.send(ANNOUNCE);
    expect(rig.hubs.last().received).toEqual([['t1', ANNOUNCE]]);
  });

  it('falls to the leader tier when there is no worker at all', async () => {
    const rig = createRig();
    rig.env.locks = rig.locks;
    rig.env.openWire = rig.network.open;
    connectHub(rig.env);
    await rig.settle();
    expect(rig.locks.isHeld('argus-hub-leader')).toBe(true);
  });

});

describe('C4 connectHub in-tab tier', { tags: ['critical'] }, () => {
  it('falls to an in-tab hub when locks or channels are missing', async () => {
    const lockOnly = createRig();
    lockOnly.env.locks = lockOnly.locks;
    const handle = connectHub(lockOnly.env);
    const got = listen(handle);
    await Promise.resolve();
    expect(lockOnly.locks.isHeld('argus-hub-leader')).toBe(false);
    expect(lockOnly.hubs.create).toHaveBeenCalledTimes(1);
    expect(got).toEqual([{ type: 'hub-hello', epoch: 77, v: 1 }]);
  });

  it('runs the in-tab hub as the last tier: messages in, hub messages out, close disposes', async () => {
    const rig = createRig();
    const handle = connectHub(rig.env);
    const got = listen(handle);
    await Promise.resolve();
    handle.send(ANNOUNCE);
    expect(rig.hubs.last().received).toEqual([['t1', ANNOUNCE]]);
    rig.hubs.last().port.post('t1', { type: 'reset', scope: 'rotp' });
    expect(got.at(-1)).toEqual({ type: 'reset', scope: 'rotp' });
    handle.close();
    expect(rig.hubs.last().dispose).toHaveBeenCalledTimes(1);
    handle.send(ANNOUNCE);
    expect(rig.hubs.last().received).toHaveLength(1);
  });

  it('sends bye on page hide only when there are no locks to tell the tab is gone', () => {
    const bare = createRig();
    const handle = connectHub(bare.env);
    expect(bare.onHide).toHaveBeenCalledTimes(1);
    bare.onHide.mock.calls[0]?.[0]?.();
    expect(bare.hubs.last().received).toEqual([['t1', BYE]]);
    expect(handle.tabId).toBe('t1');
    const locked = createRig();
    locked.env.locks = locked.locks;
    locked.env.openWire = locked.network.open;
    connectHub(locked.env);
    expect(locked.onHide).not.toHaveBeenCalled();
  });
});

describe('C4 worker tab handle', { tags: ['critical'] }, () => {
  const connect = async () => {
    const rig = createRig();
    const tabSide = rig.network.open('worker-port');
    const workerSide = rig.network.open('worker-port');
    const seen: unknown[] = [];
    workerSide.listen((data) => seen.push(data));
    rig.env.locks = rig.locks;
    rig.env.openWorker = () => tabSide;
    const handle = connectHub(rig.env);
    const got = listen(handle);
    await rig.settle();
    return { rig, handle, got, workerSide, seen };
  };

  it('keeps the epoch of the hello, delivers only its own tab messages and tags what it sends', async () => {
    const { rig, handle, got, workerSide, seen } = await connect();
    workerSide.post({ tab: 't1', epoch: 9, msg: { type: 'hub-hello', epoch: 9, v: 1 } });
    workerSide.post({ tab: 'other', epoch: 9, msg: { type: 'reset', scope: 'rotp' } });
    workerSide.post({ tab: 't1', epoch: 9, msg: { type: 'reset', scope: 'rotp' } });
    workerSide.post('junk');
    await rig.settle();
    expect(got).toEqual([
      { type: 'hub-hello', epoch: 9, v: 1 },
      { type: 'reset', scope: 'rotp' },
    ]);
    handle.send(ANNOUNCE);
    await rig.settle();
    expect(seen.at(-1)).toEqual({ tab: 't1', epoch: 9, msg: ANNOUNCE });
  });

  it('beats every ten seconds, and on close says bye, stops beating and releases its lock', async () => {
    const { rig, handle, seen } = await connect();
    await vi.advanceTimersByTimeAsync(20_000);
    await rig.settle();
    expect(seen.filter((data) => JSON.stringify(data).includes('beat'))).toHaveLength(3);
    handle.close();
    await rig.settle();
    expect(seen.at(-1)).toEqual({ tab: 't1', epoch: 0, msg: BYE });
    expect(rig.locks.isHeld('argus-tab-t1')).toBe(false);
    await vi.advanceTimersByTimeAsync(30_000);
    await rig.settle();
    expect(seen.at(-1)).toEqual({ tab: 't1', epoch: 0, msg: BYE });
  });
});
