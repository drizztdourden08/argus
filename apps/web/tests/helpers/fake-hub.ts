/* @layer renderer-app @kind test */
import { vi } from 'vitest';
import type { HubPort } from '../../src/data/hub/hub-port.type';
import type { HubLike, LeaderDeps } from '../../src/data/hub/hub-transport.type';
import type { TabMessage } from '../../src/data/hub/hub.type';

interface FakeHub extends HubLike {
  port: HubPort;
  received: Array<[string, TabMessage]>;
}

const createFakeHubs = () => {
  const hubs: FakeHub[] = [];
  const create: LeaderDeps['createHub'] = (port) => {
    const received: Array<[string, TabMessage]> = [];
    const hub: FakeHub = {
      port,
      received,
      receive: (tabId, message) => received.push([tabId, message]),
      dropTab: vi.fn(),
      dispose: vi.fn(),
    };
    hubs.push(hub);
    return hub;
  };
  return { create: vi.fn(create), hubs, last: (): FakeHub => hubs[hubs.length - 1] as FakeHub };
};

export { createFakeHubs };
