/* @layer renderer-app @kind types */
import type { TreeEnv } from './review-props.type';
import type { CleanupDoc, ReviewAccount, ReviewDoc, ReviewEdit, ReviewRow } from '../../../../data/types';

type ReviewFilter = 'all' | 'differences' | 'archived';

type ChatEdit = Extract<ReviewEdit, { sessionId: string }>;

type ChatOp = 'archive' | 'unarchive' | 'delete_chat';

type EditStager = (edits: readonly ReviewEdit[]) => void;

type BulkWant = 'active' | 'archived' | 'any';

interface EditNames {
  titles: ReadonlyMap<string, string>;
  accounts: ReadonlyMap<string, string>;
}

type Decisions = Readonly<Record<string, string>>;

interface StagedEdit {
  key: number;
  edit: ReviewEdit;
}

interface ReviewFeed {
  data: ReviewDoc | null;
  loading: boolean;
  error: string | null;
  rescan: () => void;
}

interface FeedState {
  data: ReviewDoc | null;
  loading: boolean;
  error: string | null;
}

type RowActionName = 'rename' | 'delete_folder' | 'new_folder' | 'move' | 'archive' | 'unarchive' | 'delete_chat';

interface RowAction {
  name: RowActionName;
  account: string;
  id: string;
  title: string;
}

type RowActFn = (action: RowAction) => void;

interface MenuContext {
  row: ReviewRow;
  account: string;
  act: RowActFn;
}

interface SelectState {
  checked: boolean;
  mixed: boolean;
}

type ReviewDialog =
  | { kind: 'rename'; account: string; folder: string }
  | { kind: 'new'; account: string }
  | { kind: 'move'; account: string; ids: readonly string[] }
  | { kind: 'cleanup' }
  | { kind: 'apply' };

type Selection = Readonly<Record<string, readonly string[]>>;

interface DecisionState {
  decisions: Decisions;
  choose: (rowId: string, choice: string, fallback: string) => void;
}

interface EditStage {
  items: readonly StagedEdit[];
  add: (edits: readonly ReviewEdit[]) => void;
  remove: (key: number) => void;
}

interface SelectionState {
  selection: Selection;
  set: (account: string, ids: readonly string[], on: boolean) => void;
  clear: (account: string) => void;
}

interface DialogState {
  dialog: ReviewDialog | null;
  open: (next: ReviewDialog) => void;
  close: () => void;
}

interface CollapseState {
  collapsed: ReadonlySet<string>;
  toggle: (rowId: string) => void;
}

interface CleanupState {
  days: number;
  loading: boolean;
  error: string | null;
  found: CleanupDoc | null;
  picked: ReadonlySet<string>;
  setDays: (days: number) => void;
  find: () => void;
  toggle: (key: string) => void;
}

interface ApplyState {
  busy: boolean;
  apply: (decisions: Decisions, edits: readonly ReviewEdit[]) => void;
}

interface ReviewDeck {
  rows: readonly ReviewRow[];
  accounts: readonly ReviewAccount[];
  shown: readonly ReviewRow[];
  filter: ReviewFilter;
  setFilter: (next: ReviewFilter) => void;
  choices: DecisionState;
  staged: EditStage;
  edits: readonly ReviewEdit[];
  picks: SelectionState;
  dialog: DialogState;
  submit: ApplyState;
  names: EditNames;
  env: TreeEnv;
  changed: number;
  count: number;
}

interface TotalLine {
  label: string;
  count: number;
}

export type {
  ApplyState, BulkWant, ChatEdit, ChatOp, CleanupState, CollapseState, DecisionState, DialogState, Decisions, EditNames, EditStager, EditStage, FeedState, ReviewDeck, ReviewDialog, ReviewFeed, ReviewFilter, MenuContext, RowAction,
  RowActFn, RowActionName, SelectState, Selection, SelectionState, StagedEdit, TotalLine,
};
