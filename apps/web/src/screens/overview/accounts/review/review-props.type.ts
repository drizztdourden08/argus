/* @layer renderer-app @kind types */
import type { CleanupDoc, ReviewAccount, ReviewDiff, ReviewDoc, ReviewEdit, ReviewRow } from '../../../../data/types';
import type { CleanupState, Decisions, EditNames, ReviewDialog, ReviewFilter, RowAction, Selection, StagedEdit, TotalLine } from './review.type';

interface ReviewViewProps {
  onBack: () => void;
  onApplied: () => void;
}

interface ReviewBodyProps {
  data: ReviewDoc | null;
  loading: boolean;
  error: string | null;
  onRescan: () => void;
  onBack: () => void;
  onApplied: () => void;
}

interface TreeEnv {
  decisions: Decisions;
  collapsed: ReadonlySet<string>;
  selection: Selection;
  pending: ReadonlySet<string>;
  members: ReadonlyMap<string, ReadonlyMap<string, readonly string[]>>;
  now: number;
  onToggle: (rowId: string) => void;
  onSelect: (account: string, ids: readonly string[], on: boolean) => void;
  onAct: (action: RowAction) => void;
  onChoose: (rowId: string, choice: string, fallback: string) => void;
}

interface ReviewToolbarProps {
  accounts: readonly ReviewAccount[];
  counts: ReviewDoc['counts'];
  edits: number;
  filter: ReviewFilter;
  loading: boolean;
  applyCount: number;
  onFilter: (next: ReviewFilter) => void;
  onBack: () => void;
  onRescan: () => void;
  onCleanup: () => void;
  onApply: () => void;
}

interface ReviewChipsProps {
  accounts: readonly ReviewAccount[];
  counts: ReviewDoc['counts'];
  edits: number;
}

interface ReviewEmptyProps {
  loaded: boolean;
  total: number;
  shown: number;
}

interface CellNotesProps {
  archived: string | null;
  note: string | null;
  pending: boolean;
}

interface ScanStatusProps {
  loading: boolean;
}

interface PendingPanelProps {
  items: readonly StagedEdit[];
  names: EditNames;
  changed: number;
  onRemove: (key: number) => void;
}

interface BulkBarsProps {
  accounts: readonly ReviewAccount[];
  rows: readonly ReviewRow[];
  selection: Selection;
  onStage: (edits: readonly ReviewEdit[]) => void;
  onMove: (account: string, ids: readonly string[]) => void;
  onClear: (account: string) => void;
}

interface BulkBarProps {
  label: string;
  total: number;
  archive: number;
  unarchive: number;
  remove: number;
  onArchive: () => void;
  onUnarchive: () => void;
  onMove: () => void;
  onDelete: () => void;
  onClear: () => void;
}

interface ReviewTableProps {
  accounts: readonly ReviewAccount[];
  rows: readonly ReviewRow[];
  env: TreeEnv;
}

interface ReviewTableRowProps {
  accounts: readonly ReviewAccount[];
  row: ReviewRow;
  env: TreeEnv;
}

interface TreeCellProps {
  row: ReviewRow;
  account: ReviewAccount;
  env: TreeEnv;
}

interface CellLeadProps {
  row: ReviewRow;
  account: string;
  checked: boolean;
  mixed: boolean;
  env: TreeEnv;
}

interface RowGlyphProps {
  row: ReviewRow;
  collapsed: boolean;
  onToggle: () => void;
}

interface ChoiceChipProps {
  diff: ReviewDiff;
  choice: string;
  onChoose: (choice: string) => void;
}

interface ReviewDialogsProps {
  dialog: ReviewDialog | null;
  accounts: readonly ReviewAccount[];
  rows: readonly ReviewRow[];
  choiceLines: readonly TotalLine[];
  editLines: readonly TotalLine[];
  count: number;
  applying: boolean;
  onClose: () => void;
  onStage: (edits: readonly ReviewEdit[]) => void;
  onApply: () => void;
}

interface NameDialogProps {
  title: string;
  label: string;
  initial: string;
  confirmLabel: string;
  onConfirm: (name: string) => void;
  onCancel: () => void;
}

interface MoveDialogProps {
  account: string;
  count: number;
  folders: readonly string[];
  onConfirm: (folder: string | null) => void;
  onCancel: () => void;
}

interface CleanupDialogProps {
  accounts: readonly ReviewAccount[];
  onStage: (edits: readonly ReviewEdit[]) => void;
  onCancel: () => void;
}

interface CleanupListProps {
  accounts: readonly ReviewAccount[];
  found: CleanupDoc;
  picked: CleanupState['picked'];
  now: number;
  onToggle: (key: string) => void;
}

interface ApplyDialogProps {
  choiceLines: readonly TotalLine[];
  editLines: readonly TotalLine[];
  count: number;
  busy: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

interface TotalListProps {
  title: string;
  lines: readonly TotalLine[];
}

export type {
  ApplyDialogProps, BulkBarProps, BulkBarsProps, CellLeadProps, CellNotesProps, ChoiceChipProps, CleanupDialogProps, CleanupListProps, MoveDialogProps, NameDialogProps,
  PendingPanelProps, ReviewBodyProps, ReviewChipsProps, ReviewDialogsProps, ReviewEmptyProps, ReviewTableProps, ReviewTableRowProps, ReviewToolbarProps, ReviewViewProps,
  RowGlyphProps, ScanStatusProps, TotalListProps, TreeCellProps, TreeEnv,
};
