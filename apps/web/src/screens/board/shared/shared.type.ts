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

interface ChatButtonProps {
  link: string;
  label: string;
  size?: 'xs' | 'sm';
}

interface DrawerToolbarProps {
  id: string;
  order: readonly string[];
  label: string;
  wide: boolean;
  onWide: () => void;
  onOpen: (id: string) => void;
  children?: ReactNode;
}

export type { ChatButtonProps, DrawerToolbarProps, EditableMarkdownProps, MarkdownFieldEditorProps, NoticeDiffProps, NoticeRow, OpNotice, OpNoticeProps, Option, StoredFilters, Writer };
