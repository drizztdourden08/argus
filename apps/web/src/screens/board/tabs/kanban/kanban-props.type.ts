/* @layer renderer-app @kind types */
import type { ItemInput, ItemRef } from '../../../../data/kanban';
import type { PurgeScope } from './trash';
import type { TrashPanel } from './trash-props.type';
import type { AddPanel, BulkOps, ItemOps, KanbanState, NoticeOps, Selection, Threads } from './kanban.type';
import type { AddPreset, AskLink, CardData, ItemGroup, KanbanView } from './model';

interface KanbanTabProps {
  boardKey: string;
  trash: boolean;
  onTrashBack: () => void;
  onPurge: (scope: PurgeScope) => void;
}

interface KanbanToolbarProps {
  state: KanbanState;
  view: KanbanView;
  threads: Threads;
  subjects: readonly string[];
}

interface FilterRowProps {
  state: KanbanState;
  threads: Threads;
  subjects: readonly string[];
}

interface ViewRowProps {
  state: KanbanState;
  view: KanbanView;
}

interface OpNoticesProps {
  ops: NoticeOps;
}

interface SelectionBarProps {
  items: readonly ItemRef[];
  threads: Threads;
  selection: Selection;
  ops: BulkOps;
}

interface ColumnAddFormProps {
  preset: AddPreset;
  group: ItemGroup;
  threads: Threads;
  onCreate: (item: ItemInput) => void;
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
  add: AddPanel;
  focus: string;
  onOpen: (id: string) => void;
  onPurge: () => void;
}

interface KanbanColumnsProps {
  view: KanbanView;
  state: KanbanState;
  threads: Threads;
  now: number;
  selection: Selection;
  ops: ItemOps;
  add: AddPanel;
  focus: string;
  onOpen: (id: string) => void;
}

interface QuietBadgeProps {
  count: number;
}

interface ColumnAddProps {
  title: string;
  open: boolean;
  onToggle: () => void;
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

interface AskTagProps {
  link: AskLink;
}

export type {
  AskTagProps, CardFootProps, CardMenuProps, CardTagsProps, ColumnAddFormProps, ColumnAddProps, CriteriaBarProps, FilterRowProps, ItemCardProps, KanbanBodyProps, KanbanColumnsProps, KanbanTabProps, KanbanToolbarProps, OpNoticesProps, QuietBadgeProps, SelectionBarProps, ViewRowProps,
};
