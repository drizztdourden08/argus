/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { OpOutcome, OwnerOp, SentOp } from '../../../data/kanban';
import type { Pictures } from '../../../data/media';
import type { ItemImage } from '../../../data/types';

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

interface ItemMedia {
  images: readonly ItemImage[];
  pictures: Pictures;
  uploading: boolean;
  attach: (files: File[]) => Promise<ItemImage[]>;
  media: MarkdownMedia | undefined;
}

interface MarkdownMedia {
  pictures: Pictures;
  count: (text: string) => number;
  onImageFiles: (files: File[]) => Promise<string>;
}

interface MarkdownFieldEditorProps {
  media?: MarkdownMedia;
  noun: string;
  value: string;
  limit: number;
  problem: (text: string) => string | null;
  onSave: (value: string) => void;
  onCancel: () => void;
}

interface EditableMarkdownProps {
  media?: MarkdownMedia;
  noun: string;
  value: string;
  empty: string;
  limit: number;
  problem: (text: string) => string | null;
  onSave: (value: string) => void;
}

interface RecordDrawerProps {
  open: boolean;
  label: string;
  wide: boolean;
  eyebrow: ReactNode;
  title: ReactNode;
  tools?: ReactNode;
  onClose: () => void;
  children?: ReactNode;
}

interface FactSelectProps {
  label: string;
  options: readonly Option[];
  value: string;
  onPick: (value: string) => void;
}

interface SoftToolsProps {
  archived: boolean;
  onArchive: () => void;
  onDelete: () => void;
}

interface DrawerToolbarProps {
  id: string;
  previous: string | null;
  next: string | null;
  label: string;
  wide: boolean;
  onWide: () => void;
  onStep: (id: string) => void;
  link?: (id: string) => string;
  before?: ReactNode;
  children?: ReactNode;
}

export type { DrawerToolbarProps, EditableMarkdownProps, FactSelectProps, ItemMedia, MarkdownFieldEditorProps, NoticeDiffProps, NoticeRow, OpNotice, OpNoticeProps, Option, RecordDrawerProps, SoftToolsProps, StoredFilters, Writer };
