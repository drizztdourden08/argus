/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { ItemRef } from '../../../../data/kanban';
import type { WithId } from '../../../../data/react';
import type { Entry as BoardEntry, Test } from '../../../../data/types';
import type { FeedActions } from '../../OwnerFeed';
import type { CriteriaReorder, DrawerNav, ItemOps, KanbanData, RecordActions, Threads } from './kanban.type';
import type { EntryRow, TimelineRow, TodoRow as TodoEntry } from './record';

interface ItemDrawerProps {
  boardKey: string;
  item: ItemRef | null;
  tests: readonly WithId<Test>[];
  threads: Threads;
  actions: KanbanData['actions'];
  waiting: boolean;
  now: number;
  ops: ItemOps;
  nav: DrawerNav;
  wide: boolean;
  onWide: () => void;
  onClose: () => void;
}

interface ItemBodyProps {
  boardKey: string;
  item: ItemRef;
  tests: readonly WithId<Test>[];
  threads: Threads;
  actions: KanbanData['actions'];
  waiting: boolean;
  now: number;
  ops: ItemOps;
  tools: ReactNode;
}

interface DrawerNavProps {
  item: ItemRef;
  ops: ItemOps;
  nav: DrawerNav;
  wide: boolean;
  onWide: () => void;
}

interface ItemTitleProps {
  item: ItemRef;
  ops: ItemOps;
}

interface DrawerFactsProps {
  item: ItemRef;
  waiting: boolean;
  threads: Threads;
  now: number;
  ops: ItemOps;
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
  rowKey: string;
  proving: readonly string[];
  ops: ItemOps;
  reorder: CriteriaReorder;
}

interface AddLineProps {
  label: string;
  buttonLabel: string;
  limit: number;
  problemOf: (text: string) => string | null;
  onAdd: (text: string) => Promise<boolean>;
}

interface TodosSectionProps {
  item: ItemRef;
  todos: readonly TodoEntry[];
  ops: ItemOps;
  acts: RecordActions;
}

interface TodoRowProps {
  todo: TodoEntry;
  ops: ItemOps;
  acts: RecordActions;
}

interface ItemTestsProps {
  tests: readonly WithId<Test>[];
}

interface EntrySectionProps {
  title: string;
  empty: string;
  rows: readonly EntryRow[];
  feed: FeedActions;
  ops: ItemOps;
  now: number;
}

interface EntryBlockProps {
  row: EntryRow;
  feed: FeedActions;
  ops: ItemOps;
  now: number;
}

interface EntryHeadProps {
  entry: BoardEntry;
  now: number;
}

interface EntryReplyProps {
  reply: BoardEntry;
  now: number;
}

interface ActivitySectionProps {
  item: ItemRef;
  rows: readonly TimelineRow[];
  acts: RecordActions;
  now: number;
}

interface TimelineLineProps {
  row: TimelineRow;
  thread: string;
  acts: RecordActions;
  now: number;
}

interface ComposerProps {
  item: ItemRef;
  acts: RecordActions;
}

interface DrawerSectionProps {
  title: string;
  count?: number;
  children: ReactNode;
}

export type {
  ActivitySectionProps, AddLineProps, ComposerProps, CriteriaSectionProps, CriterionRowProps, DescriptionEditorProps, DescriptionSectionProps, DrawerFactsProps,
  DrawerNavProps, DrawerSectionProps, EntryBlockProps, EntryHeadProps, EntryReplyProps, EntrySectionProps, ItemBodyProps, ItemDrawerProps, ItemTestsProps, ItemTitleProps,
  TimelineLineProps, TodoRowProps, TodosSectionProps,
};
