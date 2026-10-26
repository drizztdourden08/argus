/* @layer renderer-app @kind types */
import type { BulkOps, KanbanData, NoticeOps, Selection, Threads } from './kanban.type';
import type { PurgeCounts, PurgeDraft, PurgeNotice, PurgeScope, TrashRow } from './trash';

interface TrashControl {
  open: boolean;
  count: number;
  onToggle: () => void;
}

interface TrashPanel {
  open: boolean;
  rows: readonly TrashRow[];
  selection: Selection;
  bulk: BulkOps;
}

interface PurgeNoticeViewProps {
  notice: PurgeNotice;
  ops: NoticeOps;
}

interface PurgeDialogProps {
  boardKey: string;
  data: KanbanData;
  focus: string;
  scope: PurgeScope;
  onClose: () => void;
}

interface PurgeCountsViewProps {
  counts: PurgeCounts;
}

interface PurgeScopeFieldsProps {
  draft: PurgeDraft;
  threads: Threads;
  patch: (patch: Partial<PurgeDraft>) => void;
}

interface TrashViewProps {
  rows: readonly TrashRow[];
  now: number;
  selection: Selection;
  bulk: BulkOps;
  onPurge: () => void;
}

interface TrashLineProps {
  row: TrashRow;
  now: number;
  selected: boolean;
  selection: Selection;
  order: readonly string[];
  bulk: BulkOps;
}

export type {
  PurgeCountsViewProps, PurgeDialogProps, PurgeNoticeViewProps, PurgeScopeFieldsProps, TrashControl, TrashLineProps, TrashPanel, TrashViewProps,
};
