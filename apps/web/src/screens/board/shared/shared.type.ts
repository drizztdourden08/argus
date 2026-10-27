/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { OpOutcome, OwnerOp, SentOp } from '../../../data/kanban';

type Writer = (send: () => Promise<SentOp>, done: string, undo?: (sentId: string) => void) => Promise<boolean>;

interface OpNotice {
  id: string;
  entryId: string;
  bulk: boolean;
  kind: 'conflict' | 'refused';
  op: OwnerOp;
  result: OpOutcome;
}

interface NoticeRow {
  field: string;
  mine: string;
  theirs: string;
}

interface Option {
  value: string;
  label: string;
}

interface NoticeActions {
  dismiss: (notice: OpNotice) => void;
  resend: (notice: OpNotice) => void;
}

interface OpNoticeProps {
  notice: OpNotice;
  ops: NoticeActions;
}

interface NoticeDiffProps {
  notice: OpNotice;
}

interface StoredFilters {
  query: string;
  conflict: boolean;
}

interface MarkdownFieldEditorProps {
  noun: string;
  value: string;
  limit: number;
  problem: (text: string) => string | null;
  onSave: (value: string) => void;
  onCancel: () => void;
}

interface EditableMarkdownProps {
  noun: string;
  value: string;
  empty: string;
  limit: number;
  problem: (text: string) => string | null;
  onSave: (value: string) => void;
}

interface RecordHeaderProps {
  eyebrow: ReactNode;
  title: ReactNode;
  tools?: ReactNode;
  onClose: () => void;
}

interface RecordDrawerProps extends RecordHeaderProps {
  open: boolean;
  label: string;
  wide: boolean;
  children?: ReactNode;
}

export type { EditableMarkdownProps, RecordDrawerProps, RecordHeaderProps, MarkdownFieldEditorProps, NoticeDiffProps, NoticeRow, OpNotice, OpNoticeProps, Option, StoredFilters, Writer };
