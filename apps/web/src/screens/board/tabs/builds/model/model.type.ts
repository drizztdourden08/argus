/* @layer renderer-app @kind types */
import type { WithId } from '../../../../../data/react';
import type { RunEntry } from '../../../../../data/runs';
import type { Build, LaunchDef, Launcher } from '../../../../../data/types';

type LauncherDoc = WithId<Launcher>;
type BuildDoc = WithId<Build>;
type LaunchDefDoc = WithId<LaunchDef>;

interface LauncherUsage {
  every: string;
  legacy: boolean;
  lastUsed: string;
  stale: boolean;
}

interface LauncherView {
  launcher: LauncherDoc;
  def: LaunchDefDoc | null;
  usage: LauncherUsage | null;
  uses: string[];
  joined: BuildDoc | null;
  playable: boolean;
  runs: RunEntry[];
  shown: RunEntry | null;
}

interface ViewInput {
  launchers: readonly LauncherDoc[];
  defs: readonly LaunchDefDoc[];
  runs: readonly RunEntry[];
  builds: readonly BuildDoc[];
  now: number;
}

interface LauncherViews {
  live: LauncherView[];
  retired: LauncherView[];
  staleCount: number;
  hasMaster: boolean;
}

interface RunGroups {
  live: RunEntry[];
  finished: RunEntry[];
}

interface TerminalInput {
  enabled: boolean;
  placeholder: string;
}

export type { BuildDoc, LaunchDefDoc, LauncherDoc, LauncherUsage, LauncherView, LauncherViews, RunGroups, TerminalInput, ViewInput };
