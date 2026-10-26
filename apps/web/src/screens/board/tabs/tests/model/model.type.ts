/* @layer renderer-app @kind types */
import type { OpNotice } from '../../kanban/model';
import type { WithId } from '../../../../../data/react';
import type { Build, LaunchDef, Launcher, Test, TestItem, Wake } from '../../../../../data/types';

type TestNotice = OpNotice;

type TestDoc = WithId<Test>;
type BuildDoc = WithId<Build>;
type LauncherDoc = WithId<Launcher>;
type LaunchDefDoc = WithId<LaunchDef>;
type ItemDoc = WithId<TestItem>;
type WakeDoc = WithId<Wake>;

type LaunchState = 'linked' | 'retired' | 'missing' | 'legacy' | 'none';

type TestGroup = 'build' | 'item' | 'thread';

type TestOrder = 'rank' | 'updated';

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

interface BundleView {
  group: BundleGroup;
  launch: LaunchInfo;
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
  narrowed?: boolean;
}

interface TestFilters {
  query: string;
  threads: string[];
  builds: string[];
  nobuild: boolean;
  status: string;
  subject: string;
  needs: boolean;
  conflict: boolean;
  archived: boolean;
}

interface TestPrefs {
  group: TestGroup;
  order: TestOrder;
  showPassed: boolean;
  showClosed: boolean;
}

interface StoredView {
  filters: TestFilters;
  prefs: TestPrefs;
}

interface RawStored {
  filters: string | null;
  group: string | null;
  passed: string | null;
  closed: string | null;
}

interface GroupCard {
  key: string;
  item: ItemDoc | null;
  tests: TestDoc[];
  counts: Counts;
}

interface ViewInput {
  tests: readonly TestDoc[];
  builds: readonly BuildDoc[];
  items: readonly ItemDoc[];
  launchers: readonly LauncherDoc[];
  defs: readonly LaunchDefDoc[];
  handed: string | null;
  filters: TestFilters;
  prefs: TestPrefs;
  conflicted: ReadonlySet<string>;
}

interface TestsView {
  bundles: BundleView[];
  cards: GroupCard[];
  closed: ClosedGroup[];
  toRun: number;
  total: number;
  filtered: boolean;
  closedCount: number;
}

interface WakeChip {
  tone: 'success' | 'warning' | 'neutral';
  pulse: boolean;
  label: string;
  title: string;
}

interface WakeInput {
  test: TestDoc;
  wakes: readonly WakeDoc[];
  latest: ReadonlyMap<string, WakeDoc>;
  now: number;
}

interface EditChange {
  title: string;
  covers: string;
  build: string;
  mode: string;
}

export type {
  BuildDoc, BundleGroup, BundleView, ClosedGroup, Counts, EditChange, GroupCard, GroupInput, ItemDoc, LaunchDefDoc, LauncherDoc, LaunchInfo, RawStored, StoredView,
  TestDoc, TestFilters, TestGroup, TestNotice, TestOrder, TestPrefs, TestsView, ViewInput, WakeChip, WakeDoc, WakeInput,
};
