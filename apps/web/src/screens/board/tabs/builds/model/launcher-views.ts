/* @layer renderer-app @kind logic */
import { isLiveRun, runsOf } from '../../../../../data/runs';
import { MASTER } from '../builds.constants';
import { launcherUsage } from './launcher-usage';
import type { BuildDoc, LauncherDoc, LauncherView, LauncherViews, ViewInput } from './model.type';
import { sortLaunchers } from './sort-launchers';
import { sortRetired } from './sort-retired';

const usesOf = (launcher: LauncherDoc, builds: readonly BuildDoc[]): string[] =>
  builds.filter((build) => build.launcher === launcher.name && !build.closedAt).map((build) => build.id);

const joinedOf = (launcher: LauncherDoc, builds: readonly BuildDoc[]): BuildDoc | null => {
  if (!launcher.joined || launcher.name === MASTER) return null;
  return builds.find((build) => build.id === launcher.joined) ?? null;
};

const viewOf = (launcher: LauncherDoc, input: ViewInput): LauncherView => {
  const retired = Boolean(launcher.retiredAt);
  const def = retired ? null : input.defs.find((candidate) => candidate.launcher === launcher.name) ?? null;
  const joined = joinedOf(launcher, input.builds);
  const runs = def ? runsOf(input.runs, def.name) : [];
  return {
    launcher,
    def,
    usage: launcherUsage(launcher, input.runs, input.now),
    uses: usesOf(launcher, input.builds),
    joined,
    playable: !retired && launcher.stage !== 'closed' && joined?.status !== 'draft',
    runs,
    shown: runs.find(isLiveRun) ?? runs[0] ?? null,
  };
};

const launcherViews = (input: ViewInput): LauncherViews => {
  const live = sortLaunchers(input.launchers.filter((launcher) => !launcher.retiredAt)).map((launcher) => viewOf(launcher, input));
  const retired = sortRetired(input.launchers.filter((launcher) => launcher.retiredAt)).map((launcher) => viewOf(launcher, input));
  return {
    live,
    retired,
    staleCount: live.filter((view) => view.usage?.stale).length,
    hasMaster: live.some((view) => view.launcher.name === MASTER),
  };
};

export { launcherViews };
