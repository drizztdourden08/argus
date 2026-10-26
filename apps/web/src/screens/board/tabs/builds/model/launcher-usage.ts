/* @layer renderer-app @kind logic */
import { DEFAULT_USE_EVERY, DEFAULT_USE_EVERY_MS } from '../builds.constants';
import type { LauncherDoc, LauncherUsage } from './model.type';
import type { RunEntry } from '../../../../../data/runs';

const stamp = (iso: string | null | undefined): number => Date.parse(iso ?? '') || 0;

const lastUse = (launcher: LauncherDoc, runs: readonly RunEntry[]): number =>
  runs.reduce(
    (top, run) => (run.name === `launcher-${launcher.name}` ? Math.max(top, stamp(run.queuedAt)) : top),
    stamp(launcher.lastUsedAt),
  );

const launcherUsage = (launcher: LauncherDoc, runs: readonly RunEntry[], now: number): LauncherUsage | null => {
  if (!launcher.standalone || launcher.retiredAt) return null;
  const every = Number(launcher.useEveryMs) || DEFAULT_USE_EVERY_MS;
  const last = lastUse(launcher, runs);
  const held = stamp(launcher.keptUntil) > now;
  return {
    every: launcher.useEvery ?? DEFAULT_USE_EVERY,
    legacy: !launcher.useEvery,
    lastUsed: new Date(last).toISOString(),
    stale: !held && Math.max(0, now - last) > every,
  };
};

export { launcherUsage };
