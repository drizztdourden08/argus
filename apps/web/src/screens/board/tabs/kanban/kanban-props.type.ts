/* @layer renderer-app @kind types */
import type { ItemInput, ItemRef } from '../../../../data/kanban';
import type { PurgeScope } from './trash';
import type { TrashPanel } from './trash-props.type';
import type { AddPanel, BulkOps, ItemOps, KanbanState, NoticeOps, Selection, Threads } from './kanban.type';
import type { AddPreset, CardData, CardProgress, ItemGroup, KanbanView } from './model';
import type { LayoutState, TreeSource } from '../kanban-tree';

interface KanbanTabProps {
  boardKey: string;
  trash: boolean;
  onTrashBack: () => void;
  onPurge: (scope: PurgeScope) => void;
}

interface KanbanToolbarProps {
  boardKey: string;
  state: KanbanState;
  view: KanbanView;
  items: readonly ItemRef[];
  threads: Threads;
  selection: Selection;
  layout: LayoutState;
}

interface FilterRowProps {
  boardKey: string;
  state: KanbanState;
  items: readonly ItemRef[];
  threads: Threads;
}

interface FilterChecksProps {
  state: KanbanState;
}

interface ViewRowProps {
  state: KanbanState;
  view: KanbanView;
  selection: Selection;
  layout: LayoutState;
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
  boardKey: string;
  view: KanbanView;
  state: KanbanState;
  threads: Threads;
  now: number;
  selection: Selection;
  ops: ItemOps;
  trash: TrashPanel;
  add: AddPanel;
  focus: string;
  layout: LayoutState;
  source: TreeSource;
  onOpen: (id: string) => void;
  onPurge: () => void;
}

interface KanbanColumnsProps {
  boardKey: string;
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

interface ParentHeaderProps {
  parent: ItemRef;
}

interface ColumnAddProps {
  title: string;
  open: boolean;
  onToggle: () => void;
}

interface ItemCardProps {
  boardKey: string;
  card: CardData;
  group: ItemGroup;
  threads: Threads;
  now: number;
  selected: boolean;
  selecting: boolean;
  ops: ItemOps;
  onToggle: (id: string) => void;
  onMark: (shift: boolean) => void;
  onOpen: (id: string) => void;
}

interface CardTagsProps {
  card: CardData;
  now: number;
  onOpen: (id: string) => void;
}

interface AgentLineProps {
  item: ItemRef;
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

interface CardProgressRowProps {
  segments: readonly CardProgress[];
}


export type {
  AgentLineProps, CardFootProps, CardMenuProps, CardProgressRowProps, CardTagsProps, ColumnAddFormProps, ColumnAddProps, FilterChecksProps, FilterRowProps, ItemCardProps, KanbanBodyProps, KanbanColumnsProps, KanbanTabProps, KanbanToolbarProps, OpNoticesProps, ParentHeaderProps, QuietBadgeProps, SelectionBarProps, ViewRowProps,
};
