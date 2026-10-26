/* @layer renderer-app @kind types */
import type { BulkOps, KanbanData, NoticeOps, Selection, Threads } from './kanban.type';
import type { PurgeCounts, PurgeDraft, PurgeNotice, PurgeRequest, PurgeScope, TrashRow } from './trash';

interface TrashPanel {
  open: boolean;
  onBack: () => void;
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
  thread: string;
  scope: PurgeScope;
  onClose: () => void;
}

interface KanbanPurgeProps {
  boardKey: string;
  request: PurgeRequest | null;
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
  onBack: () => void;
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
  KanbanPurgeProps, PurgeCountsViewProps, PurgeDialogProps, PurgeNoticeViewProps, PurgeScopeFieldsProps, TrashLineProps, TrashPanel, TrashViewProps,
};
