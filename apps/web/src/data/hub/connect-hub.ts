/* @layer renderer-app @kind logic */
import { browserEnv } from './browser-env';
import { FallbackTab } from './fallback-tab';
import type { TabTier } from './fallback-tab.type';
import { LeaderTab } from './hub-leader';
import { TIER_LIVE_MS } from './hub-transport.constants';
import type { ConnectEnv } from './hub-transport.type';
import { InPageTab } from './in-page-tab';
import type { TabHandle } from './tab-handle.type';
import { WorkerTab } from './worker-tab';

const tiers = (env: ConnectEnv): TabTier[] => {
  const { tabId, locks, openWorker, openWire } = env;
  const list: TabTier[] = [];
  if (openWorker) list.push(() => new WorkerTab({ tabId, wire: openWorker(), locks }));
  if (locks && openWire) list.push(() => new LeaderTab({ tabId, locks, openWire, createHub: env.createHub, now: env.now }));
  return list;
};

const connectHub = (env: ConnectEnv = browserEnv()): TabHandle => {
  const handle = new FallbackTab({
    tabId: env.tabId,
    tiers: tiers(env),
    last: () => new InPageTab(env),
    health: env.health ?? null,
    waitMs: TIER_LIVE_MS,
  });
  if (!env.locks) env.onHide(() => handle.send({ type: 'bye' }));
  return handle;
};

export { connectHub };
