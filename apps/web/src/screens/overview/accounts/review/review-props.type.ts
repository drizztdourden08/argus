/* @layer renderer-app @kind types */
import type { CleanupDoc, ReviewAccount, ReviewDoc, ReviewEdit } from '../../../../data/types';
import type { CleanupState, EditNames, ReviewDialog, ReviewFilter, StagedEdit, TotalLine } from './review.type';

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
  onExpandAll: () => void;
  onCollapseAll: () => void;
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

interface ReviewDialogsProps {
  dialog: ReviewDialog | null;
  accounts: readonly ReviewAccount[];
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
  ApplyDialogProps, CellNotesProps, CleanupDialogProps, CleanupListProps, NameDialogProps, PendingPanelProps, ReviewBodyProps, ReviewChipsProps, ReviewDialogsProps,
  ReviewEmptyProps, ReviewToolbarProps, ReviewViewProps, ScanStatusProps, TotalListProps,
};
