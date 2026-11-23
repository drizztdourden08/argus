/* @layer renderer-app @kind types */
import type { useTabView } from '../../BoardTabsRow';
import type { WorktreesMeta } from '../../../../data/worktrees';
import type { Notice, Panel, PanelKind, WorktreeFilters, WorktreeGroup, WorktreeLayout, WorktreeOrder, WorktreeRow } from './model';

type WorktreeView = ReturnType<typeof useTabView<WorktreeGroup, WorktreeOrder, WorktreeFilters>>;

type Attempt<T> = { ok: true; value: T } | { ok: false; error: unknown };

interface Guard {
  busy: boolean;
  attempt: <T>(task: () => Promise<T>) => Promise<Attempt<T> | null>;
}

interface WorktreesData {
  ready: boolean;
  meta: WorktreesMeta | null;
  rows: WorktreeRow[];
}

interface TabPrefs {
  needOnly: boolean;
  setNeedOnly: (next: boolean) => void;
  compact: boolean;
  setCompact: (next: boolean) => void;
  layout: WorktreeLayout;
  setLayout: (layout: WorktreeLayout) => void;
  view: WorktreeView;
}

interface Notices {
  notice: Notice | null;
  dismissed: ReadonlySet<string>;
  show: (notice: Notice) => void;
  clear: () => void;
  dismissJobs: (ids: readonly string[]) => void;
}

interface PanelControls {
  panel: Panel | null;
  toggle: (row: WorktreeRow, kind: PanelKind) => void;
  close: () => void;
  patch: (change: Partial<Panel>) => void;
}

interface AskThread {
  busy: boolean;
  send: (row: WorktreeRow) => Promise<void>;
}

interface Removal {
  busy: boolean;
  confirm: (row: WorktreeRow) => Promise<void>;
}

interface BatchState {
  error: string | null;
}

interface BatchControls {
  state: BatchState | null;
  busy: boolean;
  toggle: () => void;
  cancel: () => void;
  confirm: () => Promise<void>;
}

interface WorktreeMenuInput {
  meta: WorktreesMeta | null;
  controls: PanelControls;
  open: () => void;
  code: string;
  label: string;
  keep: () => void;
  link: () => void;
  unlink: () => void;
}

export type {
  Attempt, AskThread, BatchControls, BatchState, Guard, Notices, PanelControls, Removal, TabPrefs, WorktreeMenuInput, WorktreesData,
};
