/* @layer renderer-app @kind types */
import type { TreeEnv } from './review-tree-props.type';
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

type RowActionName = 'rename' | 'delete_folder' | 'new_folder' | 'archive' | 'unarchive' | 'delete_chat';

type Side = 'left' | 'right' | 'skip';

type ChooseFn = (rowId: string, choice: string, fallback: string) => void;

interface SideState {
  enabled: boolean;
  active: boolean;
}

interface Pair {
  left: string;
  right: string;
}

interface ChooseSideArgs {
  rows: readonly ReviewRow[];
  side: Side;
  pair: Pair;
  revert: boolean;
  onChoose: ChooseFn;
}

interface RowView {
  present: boolean;
  kids: boolean;
  collapsed: boolean;
  archived: string | null;
  note: string | null;
  count: number | null;
}

interface RowAction {
  name: RowActionName;
  account: string;
  id: string;
  title: string;
}

type RowActFn = (action: RowAction) => void;

type ReviewDialog =
  | { kind: 'rename'; account: string; folder: string }
  | { kind: 'new'; account: string }
  | { kind: 'cleanup' }
  | { kind: 'apply' };

interface DecisionState {
  decisions: Decisions;
  choose: (rowId: string, choice: string, fallback: string) => void;
}

interface EditStage {
  items: readonly StagedEdit[];
  add: (edits: readonly ReviewEdit[]) => void;
  remove: (key: number) => void;
}

interface DialogState {
  dialog: ReviewDialog | null;
  open: (next: ReviewDialog) => void;
  close: () => void;
}

interface CollapseState {
  collapsed: ReadonlySet<string>;
  toggle: (rowId: string) => void;
  setAll: (ids: ReadonlySet<string>) => void;
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
  dialog: DialogState;
  submit: ApplyState;
  names: EditNames;
  env: TreeEnv;
  hovered: string | null;
  setHovered: (rowId: string | null) => void;
  expandAll: () => void;
  collapseAll: () => void;
  changed: number;
  count: number;
}

interface TotalLine {
  label: string;
  count: number;
}

export type {
  ApplyState, BulkWant, ChatEdit, ChatOp, ChooseFn, ChooseSideArgs, CleanupState, CollapseState, DecisionState, DialogState, Decisions, EditNames, EditStager, EditStage, FeedState, Pair, ReviewDeck, ReviewDialog, ReviewFeed,
  ReviewFilter, RowView, RowAction, RowActFn, Side, SideState, StagedEdit, TotalLine,
};
