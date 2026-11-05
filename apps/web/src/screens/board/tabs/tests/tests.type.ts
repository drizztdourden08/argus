/* @layer renderer-app @kind types */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { PostResult } from '../../../../data/api';
import type { Entry, SoftKind, ThreadChoice } from '../../../../data/kanban';
import type { TestOutcome } from '../../../../data/posts';
import type { WithId } from '../../../../data/react';
import type { RunEntry } from '../../../../data/runs';
import type { Build, LaunchDef, Launcher, Test, TestItem, Wake } from '../../../../data/types';
import type { EditChange, FocusOption, FocusScope, ItemDoc, TestDoc, TestFilters, TestGroup, TestNotice, TestOrder, TestPrefs, TestsView, WakeDoc } from './model';

interface TestsData {
  ready: boolean;
  tests: WithId<Test>[];
  builds: WithId<Build>[];
  launchers: WithId<Launcher>[];
  defs: WithId<LaunchDef>[];
  runs: RunEntry[];
  items: WithId<TestItem>[];
  threads: WithId<ThreadChoice>[];
  entries: WithId<Entry>[];
  wakes: WithId<Wake>[];
  handed: string | null;
}

interface TestsState {
  filters: TestFilters;
  prefs: TestPrefs;
  patchFilters: (patch: Partial<TestFilters>) => void;
  clearFilters: () => void;
  setGroup: (group: TestGroup) => void;
  setOrder: (order: TestOrder) => void;
  setDirection: (direction: SortDirection) => void;
  setShowPassed: (show: boolean) => void;
  setShowClosed: (show: boolean) => void;
}

interface TestsFocus {
  scope: FocusScope;
  options: FocusOption[];
  pausedCount: number;
  setFocus: (focus: string) => void;
  setShowPaused: (show: boolean) => void;
}

interface TestOps {
  edit: (test: TestDoc, set: Partial<EditChange>) => Promise<boolean>;
  move: (test: TestDoc, item: string) => Promise<boolean>;
  soft: (kind: SoftKind, test: TestDoc) => Promise<boolean>;
}

interface NoticeOps {
  notices: TestNotice[];
  dismiss: (notice: TestNotice) => void;
  resend: (notice: TestNotice) => void;
}

interface WakeLookup {
  all: WakeDoc[];
  latest: ReadonlyMap<string, WakeDoc>;
  now: number;
}

interface TestsShared {
  boardKey: string;
  ops: TestOps;
  items: ItemDoc[];
  threads: WithId<ThreadChoice>[];
  wakes: WakeLookup;
  group: TestGroup;
  order: TestOrder;
  direction: SortDirection | null;
  showPassed: boolean;
  conflicted: ReadonlySet<string>;
  folds: Folds;
  onOpen: (testId: string) => void;
  onNote: (testId: string) => void;
  onEdit: (testId: string) => void;
  onLog: (runId: string) => void;
}

interface TestMenuInput {
  test: TestDoc;
  shared: TestsShared;
  rerun: () => void;
}

interface TestsScreen {
  state: TestsState;
  view: TestsView;
  shared: TestsShared;
  notices: NoticeOps;
  visible: TestDoc[];
  focus: TestsFocus;
  noteId: string | null;
  editId: string | null;
  logId: string | null;
  closeNote: () => void;
  closeEdit: () => void;
  closeLog: () => void;
}

interface OwnerPost {
  busy: boolean;
  run: (send: () => Promise<PostResult>, done: string) => Promise<string | null>;
}

interface ProofUpload {
  uploading: boolean;
  attach: (testId: string, file: File, caption: string) => Promise<string | null>;
}

interface RecordResult {
  busy: boolean;
  record: (testId: string, outcome: TestOutcome, text: string, shot: File | null) => Promise<string | null>;
}

interface Outcome {
  open: TestOutcome | null;
  text: string;
  shot: File | null;
  error: string | null;
  busy: boolean;
  choose: (outcome: TestOutcome) => void;
  setText: (text: string) => void;
  setShot: (file: File) => void;
  removeShot: () => void;
  cancel: () => void;
  send: () => void;
}

interface NoteDraft {
  text: string;
  shot: File | null;
  error: string | null;
  busy: boolean;
  setText: (text: string) => void;
  setShot: (file: File) => void;
  removeShot: () => void;
  post: (outcome: TestOutcome) => void;
}

interface EditDraft {
  value: EditChange;
  item: string;
  problem: string | null;
  set: (field: keyof EditChange, value: string) => void;
  setItem: (item: string) => void;
  save: () => void;
}

interface TestActions {
  busy: boolean;
  rerun: (testId: string) => Promise<void>;
  hand: (tag: string | null) => Promise<void>;
}

interface Folds {
  isOpen: (key: string, fallback: boolean) => boolean;
  set: (key: string, open: boolean) => void;
}

export type {
  EditDraft, Folds, NoteDraft, NoticeOps, Outcome, OwnerPost, ProofUpload, RecordResult, TestActions, TestMenuInput, TestOps, TestsData, TestsFocus, TestsScreen, TestsShared, TestsState,
};
