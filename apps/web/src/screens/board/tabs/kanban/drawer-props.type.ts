/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { ItemRef } from '../../../../data/kanban';
import type { WithId } from '../../../../data/react';
import type { Entry as BoardEntry, Test } from '../../../../data/types';
import type { FeedActions } from '../../OwnerFeed';
import type { CriteriaReorder, ItemOps, RecordActions } from './kanban.type';
import type { AskLink, DepLink, DepView } from './model';
import type { EntryRow, TimelineRow, TodoRow as TodoEntry } from './record';

interface ItemEyebrowProps {
  item: ItemRef;
  waiting: boolean;
}

interface StatusTagProps {
  status: string;
}

interface ItemTitleProps {
  item: ItemRef;
  ops: ItemOps;
}

interface BlockerLineProps {
  blocker: string;
  link: AskLink | null;
}

interface DescriptionSectionProps {
  item: ItemRef;
  ops: ItemOps;
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
  children: ReactNode;
}

interface EmptyLineProps {
  children: ReactNode;
}

interface DependencySectionProps {
  item: ItemRef;
  now: number;
}

interface DependencyViewProps {
  link: DepLink;
  now: number;
}

interface DependencyFactsProps {
  view: DepView;
  now: number;
}

interface DependencyLinkProps {
  view: DepView;
}

interface DependencyHistoryProps {
  rows: DepView['history'];
  now: number;
}

interface CancelDependencyProps {
  dep: string;
}

export type {
  CancelDependencyProps, DependencyFactsProps, DependencyHistoryProps, DependencyLinkProps, DependencySectionProps, DependencyViewProps,
  ActivitySectionProps, AddLineProps, BlockerLineProps, ComposerProps, CriteriaSectionProps, CriterionRowProps, DescriptionSectionProps,
  DrawerSectionProps, EntryBlockProps, EntryHeadProps, EmptyLineProps, EntryReplyProps, EntrySectionProps, ItemEyebrowProps, ItemTestsProps,
  ItemTitleProps, StatusTagProps,
  TimelineLineProps, TodoRowProps, TodosSectionProps,
};
