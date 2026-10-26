/* @layer renderer-app @kind types */
import type { WithId } from '../../../../../data/react';
import type { Build, LaunchDef, Launcher, RunDoc, Test } from '../../../../../data/types';

type TestDoc = WithId<Test>;
type BuildDoc = WithId<Build>;
type LauncherDoc = WithId<Launcher>;
type LaunchDefDoc = WithId<LaunchDef>;
type RunEntry = WithId<RunDoc>;

type LaunchState = 'linked' | 'retired' | 'missing' | 'legacy' | 'none';

type RunTone = 'warning' | 'success' | 'info' | 'neutral' | 'danger';

interface Counts {
  run: number;
  pass: number;
  fail: number;
  partial: number;
  blocked: number;
}

interface LaunchInfo {
  state: LaunchState;
  name: string | null;
  cmd: string;
  cwd: string;
  def: LaunchDefDoc | null;
  gone: string;
}

interface BundleGroup {
  tag: string;
  build: BuildDoc | null;
  tests: TestDoc[];
  counts: Counts;
  handed: boolean;
  open: boolean;
}

interface ClosedGroup {
  tag: string;
  build: BuildDoc;
  tests: TestDoc[];
  counts: Counts;
}

interface GroupInput {
  tests: readonly TestDoc[];
  builds: readonly BuildDoc[];
  handed: string | null;
}

interface RunLine {
  tone: RunTone;
  label: string;
  since: string | null;
  last: string;
}

export type { BuildDoc, BundleGroup, ClosedGroup, Counts, GroupInput, LaunchDefDoc, LauncherDoc, LaunchInfo, RunEntry, RunLine, RunTone, TestDoc };
