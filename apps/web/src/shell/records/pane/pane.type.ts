/* @layer renderer-app @kind types */
import type { IconName, TabBadgeTone } from '@drizztdourden08/tessera';
import type { ComponentType, ReactNode } from 'react';
import type { ItemRef, MoveTarget } from '../../../data/kanban';
import type { WithId } from '../../../data/react';
import type { ItemField, RegistryKind } from '../../../data/registry';
import type { ContentContext } from '../../../data/content';
import type { Entry, Test } from '../../../data/types';
import type { FeedActions } from '../../../screens/board/OwnerFeed';
import type { Option } from '../../../screens/board/shared';
import type { ItemOps, KanbanData, RecordActions, Threads } from '../../../screens/board/tabs/kanban/kanban.type';
import type { AskLink } from '../../../screens/board/tabs/kanban/model';
import type { EntryRow, ItemRecord } from '../../../screens/board/tabs/kanban/record';
import type { RecordPaneState, RecordRef } from '../records.type';

type PaneDoc = Readonly<Record<string, unknown>>;

interface PaneData {
  boardKey: string;
  item: ItemRef;
  items: readonly ItemRef[];
  tests: readonly WithId<Test>[];
  allTests: readonly WithId<Test>[];
  threads: Threads;
  record: ItemRecord;
  waiting: boolean;
  blocking: AskLink | null;
  now: number;
  ops: ItemOps;
  acts: RecordActions;
  feed: FeedActions;
  open: (id: string) => void;
}

interface StageFact {
  label: string;
  options: readonly Option[];
  value: string;
  target: (value: string) => MoveTarget;
}

interface PaneBadge {
  value: string | number;
  tone?: TabBadgeTone;
  title?: string;
}

interface PaneBodyProps {
  data: PaneData;
}

interface EntryFactsProps {
  entry: Entry;
  onOpenItem: (code: string) => void;
}

interface DocFact {
  key: string;
  label: string;
  text: string;
}

interface RecordData {
  boardKey: string;
  kind: RegistryKind;
  target: RecordRef;
  code: string;
  doc: PaneDoc | null;
  now: number;
  threads: Threads;
  record: ItemRecord;
  entry: EntryRow | null;
  item: PaneData | null;
  ops: ItemOps;
  acts: RecordActions;
  feed: FeedActions;
  open: (code: string, tab?: string) => void;
}

interface RemoteDocs {
  remote: boolean;
  docs: readonly (PaneDoc & { id: string })[];
  ready: boolean;
}

type RemoteLoader = (boardKey: string) => Promise<(PaneDoc & { id: string })[]>;

interface RecordLoad {
  data: RecordData | null;
  ready: boolean;
}

interface BuildRecordInput {
  boardKey: string;
  target: RecordRef;
  kanban: KanbanData;
  other: readonly { id: string }[];
  entries: readonly Entry[];
  record: ItemRecord;
  now: number;
  ops: ItemOps;
  acts: RecordActions;
  feed: FeedActions;
  open: (code: string, tab?: string) => void;
}

interface RecordBodyProps {
  data: RecordData;
}

interface PaneTab {
  label: string;
  icon: IconName;
  Body: ComponentType<RecordBodyProps>;
  badge?: (data: RecordData) => PaneBadge | undefined;
}

interface TabPlaceholderProps {
  tab: string;
  label: string;
  data: RecordData;
}

interface RecordPaneProps {
  boardKey: string;
  target: RecordRef;
}

interface RecordPaneHostProps {
  boardKey?: string;
}

interface RecordFrameProps {
  data: RecordData;
  pane: RecordPaneState;
}

interface RecordPaneToolsProps {
  code: string;
  pane: RecordPaneState;
  previous: string | null;
  next: string | null;
  wide: boolean;
  onWide: () => void;
  children?: ReactNode;
}

interface RecordDataProps {
  data: RecordData;
}

interface PaneTabsProps {
  data: RecordData;
  tabs: readonly string[];
  active: string;
  onSelect: (tab: string) => void;
}

interface PaneCrumbsProps {
  item: ItemRef;
  items: readonly ItemRef[];
  onOpen: (id: string) => void;
}

interface PaneFactsProps {
  data: PaneData;
}

interface FieldGroupModel {
  id: string;
  label: string;
  fields: readonly string[];
}

interface FieldProps {
  name: string;
  item: ItemRef;
  ops: ItemOps;
  cx: ContentContext;
  people: readonly Option[];
}

interface FieldBodyProps {
  spec: ItemField;
  text: string;
}

interface FieldGroupProps {
  group: FieldGroupModel;
  shared: Omit<FieldProps, 'name'>;
}

interface CauseRowProps {
  kind: string;
  id: string;
  label: string;
}

interface FieldSelectProps {
  label: string;
  options: readonly Option[];
  value: string;
  onPick: (value: string) => void;
}

interface RefTagsProps {
  ids: readonly string[];
}

interface TrailEdge {
  kind: string;
  id: string;
  state: string | null;
  since: string | null;
}

interface TrailNode {
  project: string;
  item: string;
  title: string;
  type: string | null;
  lane: string | null;
  stage: string | null;
  assignee: string | null;
  edge: TrailEdge | null;
  cycle: boolean;
  children: TrailNode[];
}

type NodeOpens = { kind: 'item' | 'entry' | 'test'; id: string } | { kind: 'board'; href: string };

interface TrailTree {
  depth: number;
  open: number;
  root: TrailNode;
}

interface TrailRowProps {
  node: TrailNode;
  boardKey: string;
  now: number;
  open: (id: string) => void;
}

interface TrailLabelProps {
  node: TrailNode;
  boardKey: string;
  now: number;
}

interface TrailSectionProps {
  data: PaneData;
}

interface BlockedSectionProps {
  data: PaneData;
}

interface ChildRowProps {
  child: ItemRef;
  data: PaneData;
}

interface AddChildProps {
  data: PaneData;
}

interface CaseRowProps {
  id: string;
  data: PaneData;
  onRemove: (id: string) => void;
}

export type {
  AddChildProps, BlockedSectionProps, BuildRecordInput, CaseRowProps, CauseRowProps, ChildRowProps, DocFact, EntryFactsProps, FieldBodyProps, FieldGroupModel, FieldGroupProps, FieldProps, FieldSelectProps,
  NodeOpens, PaneBadge, PaneBodyProps, PaneCrumbsProps, PaneData, PaneDoc, PaneFactsProps, PaneTab, PaneTabsProps, RecordBodyProps, RecordData, RecordFrameProps, RecordLoad, RecordPaneHostProps,
  RecordPaneProps, RecordPaneToolsProps, RecordDataProps, RefTagsProps, RemoteDocs, RemoteLoader, StageFact, TabPlaceholderProps,
  TrailEdge, TrailLabelProps, TrailNode, TrailRowProps, TrailSectionProps, TrailTree,
};
