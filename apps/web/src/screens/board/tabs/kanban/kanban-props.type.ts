/* @layer renderer-app @kind types */
import type { CardTone, KanbanGroupFold, MenuGroup, TagCategoryColor } from '@drizztdourden08/tessera';
import type { ItemInput, ItemRef } from '../../../../data/kanban';
import type { PurgeScope } from './trash';
import type { TrashPanel } from './trash-props.type';
import type { AddPanel, AgentDoc, BulkOps, ItemOps, KanbanState, NoticeOps, Selection, Threads } from './kanban.type';
import type { AddPreset, CardData, CardProgress, ColumnLimit, ItemGroup, KanbanView } from './model';
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
  agents: readonly AgentDoc[];
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
  items: ReadonlyMap<string, ItemRef>;
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

interface LimitBadgeProps {
  count: number;
  limit: ColumnLimit;
}

interface QuietBadgeProps {
  count: number;
}

interface EpicHeaderProps {
  parent: ItemRef;
  count: number;
  menu: readonly MenuGroup[];
  fold: KanbanGroupFold;
  onOpen: (id: string) => void;
}

interface NoEpicHeaderProps {
  count: number;
  fold: KanbanGroupFold;
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
  edgeTone: CardTone | undefined;
  tint: TagCategoryColor | undefined;
  compact: boolean;
  onToggle: (id: string) => void;
  onMark: (shift: boolean) => void;
  onOpen: (id: string) => void;
}

interface CardTagsProps {
  card: CardData;
  now: number;
  onOpen: (id: string) => void;
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

interface AcListProps {
  criteria: NonNullable<ItemRef['ac']>;
}

interface CardBarsProps {
  card: CardData;
}

interface CardProgressRowProps {
  segments: readonly CardProgress[];
}

export type {
  AcListProps, CardBarsProps, CardFootProps, CardMenuProps, CardProgressRowProps, CardTagsProps, ColumnAddFormProps, ColumnAddProps, EpicHeaderProps, ItemCardProps, KanbanBodyProps, KanbanColumnsProps, KanbanTabProps, KanbanToolbarProps, LimitBadgeProps, NoEpicHeaderProps, OpNoticesProps, ParentHeaderProps, QuietBadgeProps, SelectionBarProps,
};
