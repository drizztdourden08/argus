/* @layer renderer-app @kind test */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LeaderTab } from '../../src/data/hub/hub-leader';
import type { TabMessage } from '../../src/data/hub/hub.type';
import { WorkerHost } from '../../src/data/hub/worker-host';
import { WorkerTab } from '../../src/data/hub/worker-tab';
import { lateLocks } from '../helpers/late-locks';
import { createLeaderRig, START } from '../helpers/leader-rig';

const LATE_MESSAGES: Array<[string, TabMessage]> = [
  ['presence', { type: 'presence', scope: 'argus', thread: 'e2e-3' }],
  ['want', { type: 'want', scope: 'argus', coll: 'items' }],
  ['tail', { type: 'tail', id: 't1', key: 'argus', rid: 'R-1', after: 0 }],
];

const workerRig = () => {
  const { locks, network, hubs, settle } = createLeaderRig();
  const host = new WorkerHost({ locks, createHub: hubs.create, now: () => START });
  const late = lateLocks(locks);
  host.connect(network.open('port-t3'));
  const tab = new WorkerTab({ tabId: 't3', wire: network.open('port-t3'), locks: late });
  return { hubs, settle, late, tab };
};

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('C4 tab lock race', { tags: ['critical'] }, () => {
  it.each(LATE_MESSAGES)(
    'worker tier: tab opens, its lock request reaches the lock manager after the worker starts watching, hello, then %s reaches the hub',
    async (_kind, message) => {
      const { hubs, settle, late, tab } = workerRig();
      const got: string[] = [];
      tab.onMessage((hubMessage) => got.push(hubMessage.type));
      await settle();
      late.arrive();
      await settle();
      expect(got).toEqual(['hub-hello']);
      tab.send(message);
      await settle();
      expect(hubs.last().dropTab).not.toHaveBeenCalled();
      expect(hubs.last().received).toContainEqual(['t3', message]);
      tab.close();
    },
  );

  it.each(LATE_MESSAGES)(
    'leader tier: follower opens, its lock request reaches the lock manager after the leader starts watching, hello, then %s reaches the hub',
    async (_kind, message) => {
      const rig = createLeaderRig();
      rig.join('a');
      await rig.settle();
      const late = lateLocks(rig.locks);
      const { network, hubs } = rig;
      const b = new LeaderTab({ tabId: 'b', locks: late, openWire: network.open, createHub: hubs.create, now: () => START });
      await rig.settle();
      late.arrive();
      await rig.settle();
      b.send(message);
      await rig.settle();
      expect(rig.hubs.last().dropTab).not.toHaveBeenCalled();
      expect(rig.hubs.last().received).toContainEqual(['b', message]);
      b.close();
    },
  );

  it('a tab that closes before its lock is held never reaches the hub', async () => {
    const { hubs, settle, late, tab } = workerRig();
    tab.close();
    late.arrive();
    await settle();
    expect(hubs.hubs).toHaveLength(0);
  });
});
