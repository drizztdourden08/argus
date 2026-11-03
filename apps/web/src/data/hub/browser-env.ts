/* @layer renderer-app @kind logic */
import { openBroadcastWire } from './broadcast-wire';
import { Hub } from './hub';
import { HUB_WORKER_NAME } from './hub-transport.constants';
import type { ConnectEnv, Wire } from './hub-transport.type';
import { newHubTabId } from './hub-tab-id';
import { portWire } from './port-wire';

const openSharedWorker = (): Wire =>
  portWire(new SharedWorker(new URL('./hub-worker.ts', import.meta.url), { type: 'module', name: HUB_WORKER_NAME }).port);

const browserEnv = (): ConnectEnv => ({
  tabId: newHubTabId(),
  locks: 'locks' in navigator ? navigator.locks : null,
  openWorker: typeof SharedWorker === 'undefined' ? null : openSharedWorker,
  openWire: typeof BroadcastChannel === 'undefined' ? null : openBroadcastWire,
  createHub: (port) => new Hub({ port }),
  now: Date.now,
  onHide: (listener) => globalThis.addEventListener('pagehide', listener),
});

export { browserEnv };
