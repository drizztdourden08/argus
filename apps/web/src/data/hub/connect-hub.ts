/* @layer renderer-app @kind logic */
import { browserEnv } from './browser-env';
import { LeaderTab } from './hub-leader';
import type { ConnectEnv, Wire } from './hub-transport.type';
import { InPageTab } from './in-page-tab';
import type { TabHandle } from './tab-handle.type';
import { WorkerTab } from './worker-tab';

const openWorker = (env: ConnectEnv): Wire | null => {
  if (!env.openWorker) return null;
  try {
    return env.openWorker();
  } catch {
    return null;
  }
};

const pick = (env: ConnectEnv): TabHandle => {
  const wire = openWorker(env);
  if (wire) return new WorkerTab({ tabId: env.tabId, wire, locks: env.locks });
  const { locks, openWire } = env;
  if (locks && openWire) return new LeaderTab({ tabId: env.tabId, locks, openWire, createHub: env.createHub, now: env.now });
  return new InPageTab(env);
};

const connectHub = (env: ConnectEnv = browserEnv()): TabHandle => {
  const handle = pick(env);
  if (!env.locks) env.onHide(() => handle.send({ type: 'bye' }));
  return handle;
};

export { connectHub };
