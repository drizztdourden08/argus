/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { ItemInput, ItemRef } from '../../../../data/kanban';
import type { WithId } from '../../../../data/react';
import type { Test } from '../../../../data/types';
import type { BulkOps, ItemOps, KanbanState, NoticeOps, Selection, Threads } from './kanban.type';
import type { CardData, EditableField, ItemGroup, KanbanView, OpNotice } from './model';

interface KanbanTabProps {
  boardKey: string;
}

interface KanbanToolbarProps {
  state: KanbanState;
  view: KanbanView;
  threads: Threads;
  subjects: readonly string[];
  onNew: () => void;
}

interface FilterRowProps {
  state: KanbanState;
  threads: Threads;
  subjects: readonly string[];
}

interface ViewRowProps {
  state: KanbanState;
  view: KanbanView;
  onNew: () => void;
}

interface OpNoticesProps {
  ops: NoticeOps;
}

interface OpNoticeProps {
  notice: OpNotice;
  ops: NoticeOps;
}

interface NoticeDiffProps {
  notice: OpNotice;
}

interface SelectionBarProps {
  items: readonly ItemRef[];
  threads: Threads;
  selection: Selection;
  ops: BulkOps;
}

interface AddItemDialogProps {
  open: boolean;
  threads: Threads;
  onCreate: (item: ItemInput) => void;
  onClose: () => void;
}

interface KanbanColumnsProps {
  view: KanbanView;
  state: KanbanState;
  threads: Threads;
  now: number;
  selection: Selection;
  ops: ItemOps;
  onOpen: (id: string) => void;
}

interface ItemCardProps {
  card: CardData;
  group: ItemGroup;
  threads: Threads;
  now: number;
  selected: boolean;
  ops: ItemOps;
  onToggle: (id: string) => void;
  onOpen: (id: string) => void;
}

interface CardTagsProps {
  card: CardData;
  group: ItemGroup;
  now: number;
}

interface CardFootProps {
  card: CardData;
  group: ItemGroup;
  now: number;
}

interface CardMenuProps {
  item: ItemRef;
  threads: Threads;
  ops: ItemOps;
  onOpen: (id: string) => void;
}

interface CriteriaBarProps {
  item: ItemRef;
}

interface ItemDrawerProps {
  item: ItemRef | null;
  tests: readonly WithId<Test>[];
  threads: Threads;
  now: number;
  ops: ItemOps;
  onClose: () => void;
}

interface ItemBodyProps {
  item: ItemRef;
  tests: readonly WithId<Test>[];
  threads: Threads;
  now: number;
  ops: ItemOps;
}

interface DrawerActionsProps {
  item: ItemRef;
  ops: ItemOps;
}

interface DrawerFactsProps {
  item: ItemRef;
  threads: Threads;
  now: number;
  ops: ItemOps;
}

interface InlineEditProps {
  label: string;
  value: string;
  field: EditableField;
  display: ReactNode;
  onSave: (value: string) => void;
}

interface DescriptionSectionProps {
  item: ItemRef;
  ops: ItemOps;
}

interface DescriptionEditorProps {
  value: string;
  onSave: (value: string) => void;
  onCancel: () => void;
}

interface CriteriaSectionProps {
  item: ItemRef;
  tests: readonly WithId<Test>[];
  ops: ItemOps;
}

interface CriterionRowProps {
  item: ItemRef;
  index: number;
  proving: readonly string[];
  ops: ItemOps;
}

interface CriterionControlsProps {
  item: ItemRef;
  index: number;
  ops: ItemOps;
}

interface CriterionAddProps {
  item: ItemRef;
  ops: ItemOps;
}

interface ItemTestsProps {
  tests: readonly WithId<Test>[];
}

interface DrawerSectionProps {
  title: string;
  count?: number;
  children: ReactNode;
}

export type {
  CardFootProps, CardMenuProps, CardTagsProps, CriteriaBarProps, CriteriaSectionProps, CriterionAddProps, CriterionControlsProps, CriterionRowProps, DescriptionEditorProps,
  DescriptionSectionProps, DrawerActionsProps, DrawerFactsProps, DrawerSectionProps, FilterRowProps, InlineEditProps, ItemBodyProps, ItemCardProps,
  ItemDrawerProps, ItemTestsProps, KanbanColumnsProps, KanbanTabProps, KanbanToolbarProps, AddItemDialogProps, NoticeDiffProps, OpNoticeProps, OpNoticesProps,
  SelectionBarProps, ViewRowProps,
};
