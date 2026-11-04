/* @layer renderer-app @kind types */
import type { TabHandle } from './tab-handle.type';

type HealthCheck = () => Promise<number | null>;
type TabTier = () => TabHandle;

interface FallbackOptions {
  tabId: string;
  tiers: readonly TabTier[];
  last: TabTier;
  health: HealthCheck | null;
  waitMs: number;
}

interface TierWatchDeps {
  health: HealthCheck;
  waitMs: number;
  onFail: () => void;
}

export type { FallbackOptions, HealthCheck, TabTier, TierWatchDeps };
