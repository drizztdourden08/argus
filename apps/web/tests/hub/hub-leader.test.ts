/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { MUX_VERSION } from '../../src/data/hub';
import type { HubMessage } from '../../src/data/hub';
import { ANNOUNCE, createLeaderRig, START } from '../helpers/leader-rig';

const hello = (epoch: number): HubMessage => ({ type: 'hub-hello', epoch, v: MUX_VERSION });
const reset: HubMessage = { type: 'reset', scope: 'rotp' };

describe('C4 leader tier election', { tags: ['critical'] }, () => {
  it('lets the first tab lead, runs one hub there and hands the tab its own hello', async () => {
    const rig = createLeaderRig();
    const a = rig.join('a');
    await rig.settle();
    expect(rig.hubs.hubs).toHaveLength(1);
    expect(a.got).toEqual([hello(START)]);
    a.handle.send(ANNOUNCE);
    expect(rig.hubs.last().received).toEqual([['a', ANNOUNCE]]);
    rig.hubs.last().port.post('a', reset);
    expect(a.got.at(-1)).toEqual(reset);
  });

  it('greets a follower once, carries its messages in an envelope and answers on its own channel', async () => {
    const rig = createLeaderRig();
    const bus = rig.spy('argus-hub');
    const a = rig.join('a');
    const b = rig.join('b');
    await rig.settle();
    expect(b.got).toEqual([hello(START)]);
    b.handle.send(ANNOUNCE);
    await rig.settle();
    expect(bus.seen).toContainEqual({ tab: 'b', epoch: START, msg: ANNOUNCE });
    expect(rig.hubs.last().received).toContainEqual(['b', ANNOUNCE]);
    rig.hubs.last().port.post('b', reset);
    await rig.settle();
    expect(b.got).toEqual([hello(START), reset]);
    expect(a.got).toEqual([hello(START)]);
  });

  it('hands over when the leader closes: the next tab leads, every tab hears the new epoch and re-announces', async () => {
    const rig = createLeaderRig();
    const a = rig.join('a');
    const b = rig.join('b');
    const c = rig.join('c');
    await rig.settle();
    for (const tab of [b, c]) {
      tab.handle.onMessage((message) => {
        if (message.type === 'hub-hello') tab.handle.send(ANNOUNCE);
      });
    }
    a.handle.close();
    await rig.settle();
    expect(rig.hubs.hubs).toHaveLength(2);
    expect(rig.hubs.hubs[0]?.dispose).toHaveBeenCalled();
    expect(b.got.at(-1)).toEqual(hello(START + 1));
    expect(c.got.at(-1)).toEqual(hello(START + 1));
    expect(rig.hubs.last().received).toEqual([
      ['b', ANNOUNCE],
      ['c', ANNOUNCE],
    ]);
  });

  it('hands over when the leader lock is lost without a close', async () => {
    const rig = createLeaderRig();
    rig.join('a');
    const b = rig.join('b');
    await rig.settle();
    rig.locks.crash('argus-hub-leader');
    await rig.settle();
    expect(rig.hubs.hubs).toHaveLength(2);
    expect(b.got.at(-1)).toEqual(hello(START + 1));
  });
});

describe('C4 leader tier epochs', { tags: ['critical'] }, () => {
  it('ignores messages of an older epoch on the leader and greets the sender instead', async () => {
    const rig = createLeaderRig();
    rig.join('a');
    await rig.settle();
    const bus = rig.spy('argus-hub');
    const inbox = rig.spy('argus-hub:z');
    bus.wire.post({ tab: 'z', epoch: START - 5, msg: { type: 'want', scope: 'rotp', coll: 'items' } });
    bus.wire.post({ tab: 'z', epoch: START - 5, msg: ANNOUNCE });
    await rig.settle();
    expect(rig.hubs.last().received).toEqual([]);
    expect(inbox.seen).toEqual([{ tab: 'z', epoch: START, msg: hello(START) }]);
  });

  it('ignores a hello and any message of an older epoch on a follower', async () => {
    const rig = createLeaderRig();
    rig.join('a');
    const b = rig.join('b');
    await rig.settle();
    const inbox = rig.spy('argus-hub:b');
    inbox.wire.post({ tab: 'b', epoch: START - 1, msg: hello(START - 1) });
    inbox.wire.post({ tab: 'b', epoch: START - 1, msg: reset });
    inbox.wire.post({ tab: 'b', epoch: START + 4, msg: reset });
    await rig.settle();
    expect(b.got).toEqual([hello(START)]);
    inbox.wire.post({ tab: 'b', epoch: START, msg: reset });
    await rig.settle();
    expect(b.got).toEqual([hello(START), reset]);
  });

  it('ignores what is not an envelope', async () => {
    const rig = createLeaderRig();
    rig.join('a');
    await rig.settle();
    const bus = rig.spy('argus-hub');
    bus.wire.post({ hello: 'there' });
    bus.wire.post(null);
    bus.wire.post({ tab: 'q', epoch: START, msg: { type: 'nope' } });
    await rig.settle();
    expect(rig.hubs.last().received).toEqual([]);
  });
});

describe('C4 leader tier tab liveness', { tags: ['critical'] }, () => {
  it('drops a follower at once when its tab lock is released', async () => {
    const rig = createLeaderRig();
    rig.join('a');
    const b = rig.join('b');
    await rig.settle();
    b.handle.send(ANNOUNCE);
    await rig.settle();
    expect(rig.locks.queued('argus-tab-b')).toBe(1);
    rig.locks.crash('argus-tab-b');
    await rig.settle();
    expect(rig.hubs.last().dropTab).toHaveBeenCalledWith('b');
    expect(rig.network.size('argus-hub:b')).toBe(1);
  });

  it('forgets a follower that says bye and stops watching its lock', async () => {
    const rig = createLeaderRig();
    rig.join('a');
    const b = rig.join('b');
    await rig.settle();
    b.handle.send(ANNOUNCE);
    await rig.settle();
    b.handle.close();
    await rig.settle();
    expect(rig.hubs.last().received).toContainEqual(['b', { type: 'bye' }]);
    expect(rig.locks.queued('argus-tab-b')).toBe(0);
    expect(rig.network.size('argus-hub:b')).toBe(0);
  });

  it('disposes its hub and stops following when the leader closes', async () => {
    const rig = createLeaderRig();
    const a = rig.join('a');
    await rig.settle();
    a.handle.close();
    await rig.settle();
    expect(rig.hubs.last().dispose).toHaveBeenCalledTimes(1);
    expect(rig.locks.isHeld('argus-hub-leader')).toBe(false);
    expect(rig.locks.isHeld('argus-tab-a')).toBe(false);
  });
});
