/* @layer renderer-app @kind types */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { RecordGroups } from '../../../../../shell';
import type { OpNotice } from '../../../shared';
import type { WithId } from '../../../../../data/react';
import type { LaunchDef, Launcher, Test, TestItem, Wake } from '../../../../../data/types';

type TestNotice = OpNotice;

type TestDoc = WithId<Test>;
type LauncherDoc = WithId<Launcher>;
type LaunchDefDoc = WithId<LaunchDef>;
type ItemDoc = WithId<TestItem>;
type WakeDoc = WithId<Wake>;

type LaunchState = 'linked' | 'retired' | 'missing' | 'none';

type TestGroup = 'run' | 'plan' | 'item' | 'thread';

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
  note: string;
  updatedAt: string;
}

interface RunView {
  item: ItemDoc;
  plan: ItemDoc | null;
  build: string | null;
  stage: string;
  handed: boolean;
  tests: TestDoc[];
  counts: Counts;
  launch: LaunchInfo;
  open: boolean;
}

interface TestFilters {
  query: string;
  threads: string[];
  runs: string[];
  status: string;
  subject: string;
  plan: string;
  needs: boolean;
  conflict: boolean;
  archived: boolean;
}

interface TestPrefs {
  group: TestGroup;
  order: TestOrder;
  direction: SortDirection | null;
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
  direction?: string | null;
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
  items: readonly ItemDoc[];
  launchers: readonly LauncherDoc[];
  defs: readonly LaunchDefDoc[];
  filters: TestFilters;
  prefs: TestPrefs;
  conflicted: ReadonlySet<string>;
}

interface PlanView {
  item: ItemDoc;
  cases: string[];
  counts: Counts;
  live: boolean;
  closed: boolean;
}

interface TestsView {
  plans: PlanView[];
  byPlan: RecordGroups<TestDoc>;
  runs: RunView[];
  cards: GroupCard[];
  closed: RunView[];
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

interface TestChipSet {
  item: string;
  thread: string;
  auto: boolean;
  conflict: boolean;
  archived: boolean;
}

interface FocusScope {
  focus: string;
  paused: ReadonlySet<string>;
  showPaused: boolean;
}

interface FocusOption {
  value: string;
  label: string;
  paused: boolean;
}

interface EditChange {
  title: string;
  covers: string;
  mode: string;
}

export type {
  Counts, EditChange, FocusOption, FocusScope, GroupCard, ItemDoc, LaunchDefDoc, LauncherDoc, LaunchInfo, PlanView, RawStored, RunView, StoredView,
  TestChipSet, TestDoc, TestFilters, TestGroup, TestNotice, TestOrder, TestPrefs, TestsView, ViewInput, WakeChip, WakeDoc, WakeInput,
};
