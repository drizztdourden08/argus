/* @layer renderer-app @kind test */
import { LeaderTab } from '../../src/data/hub/hub-leader';
import type { HubMessage, TabMessage } from '../../src/data/hub/hub.type';
import { createFakeHubs } from './fake-hub';
import { FakeLocks } from './fake-locks';
import { FakeNetwork } from './fake-network';

const START = 1000;

const ANNOUNCE: TabMessage = {
  type: 'announce',
  tabId: 'x',
  v: 1,
  wants: [{ scope: 'rotp', coll: 'items' }],
  presences: [],
  tails: [],
};

const createLeaderRig = () => {
  const locks = new FakeLocks();
  const network = new FakeNetwork();
  const hubs = createFakeHubs();
  const settle = async (): Promise<void> => {
    for (let round = 0; round < 12; round += 1) {
      await Promise.resolve();
      network.flush();
    }
  };
  const join = (tabId: string) => {
    const handle = new LeaderTab({ tabId, locks, openWire: network.open, createHub: hubs.create, now: () => START });
    const got: HubMessage[] = [];
    handle.onMessage((message) => got.push(message));
    return { handle, got };
  };
  const spy = (name: string) => {
    const seen: unknown[] = [];
    const wire = network.open(name);
    wire.listen((data) => seen.push(data));
    return { wire, seen };
  };
  return { locks, network, hubs, settle, join, spy };
};

export { ANNOUNCE, createLeaderRig, START };
