/* @layer renderer-app @kind types */
import type { ItemInput, ItemRef } from '../../../../data/kanban';
import type { TrashControl, TrashPanel } from './trash-props.type';
import type { BulkOps, FocusState, ItemOps, KanbanData, KanbanDialog, KanbanState, NoticeOps, Selection, Threads } from './kanban.type';
import type { AddPreset, CardData, FocusChoices, ItemGroup, KanbanView, OpNotice } from './model';

interface KanbanTabProps {
  boardKey: string;
}

interface KanbanToolbarProps {
  state: KanbanState;
  view: KanbanView;
  threads: Threads;
  subjects: readonly string[];
  focus: FocusState;
  choices: FocusChoices;
  trash: TrashControl;
  onNew: () => void;
  onPurge: () => void;
}

interface FilterRowProps {
  state: KanbanState;
  threads: Threads;
  subjects: readonly string[];
}

interface FocusRowProps {
  focus: FocusState;
  choices: FocusChoices;
}

interface ViewRowProps {
  state: KanbanState;
  view: KanbanView;
  trash: TrashControl;
  onNew: () => void;
  onPurge: () => void;
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
  preset: AddPreset;
  threads: Threads;
  onCreate: (item: ItemInput) => void;
  onClose: () => void;
}

interface KanbanDialogsProps {
  boardKey: string;
  data: KanbanData;
  focus: string;
  ops: ItemOps;
  dialog: KanbanDialog;
  onClose: () => void;
}

interface KanbanBodyProps {
  view: KanbanView;
  state: KanbanState;
  threads: Threads;
  now: number;
  selection: Selection;
  ops: ItemOps;
  trash: TrashPanel;
  onOpen: (id: string) => void;
  onAdd: (column: string) => void;
  onPurge: () => void;
}

interface KanbanColumnsProps {
  view: KanbanView;
  state: KanbanState;
  threads: Threads;
  now: number;
  selection: Selection;
  ops: ItemOps;
  onOpen: (id: string) => void;
  onAdd: (column: string) => void;
}

interface QuietBadgeProps {
  count: number;
}

interface ColumnAddProps {
  title: string;
  onAdd: () => void;
}

interface ItemCardProps {
  card: CardData;
  group: ItemGroup;
  threads: Threads;
  now: number;
  selected: boolean;
  ops: ItemOps;
  onToggle: (id: string) => void;
  onMark: (shift: boolean) => void;
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

export type {
  AddItemDialogProps, CardFootProps, CardMenuProps, CardTagsProps, ColumnAddProps, CriteriaBarProps, FilterRowProps, FocusRowProps, ItemCardProps, KanbanBodyProps, KanbanColumnsProps, KanbanDialogsProps, KanbanTabProps, KanbanToolbarProps, NoticeDiffProps, OpNoticeProps, OpNoticesProps, QuietBadgeProps, SelectionBarProps, ViewRowProps,
};
