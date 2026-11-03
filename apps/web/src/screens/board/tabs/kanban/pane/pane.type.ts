/* @layer renderer-app @kind types */
import type { IconName, TabBadgeTone } from '@drizztdourden08/tessera';
import type { ComponentType } from 'react';
import type { ItemRef, MoveTarget } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import type { ItemField } from '../../../../../data/registry';
import type { ContentContext } from '../../../../../data/content';
import type { Test } from '../../../../../data/types';
import type { FeedActions } from '../../../OwnerFeed';
import type { Option } from '../../../shared';
import type { ItemOps, KanbanData, RecordActions, Threads } from '../kanban.type';
import type { AskLink } from '../model';
import type { ItemRecord } from '../record';

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

interface PaneTab {
  label: string;
  icon: IconName;
  Body: ComponentType<PaneBodyProps>;
  badge?: (data: PaneData) => PaneBadge | undefined;
}

interface ItemPaneProps {
  boardKey: string;
  item: ItemRef;
  items: readonly ItemRef[];
  tests: readonly WithId<Test>[];
  threads: Threads;
  actions: KanbanData['actions'];
  waiting: boolean;
  blocking: AskLink | null;
  now: number;
  ops: ItemOps;
  onOpen: (id: string) => void;
}

interface PaneTabsProps {
  data: PaneData;
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
  open: (id: string) => void;
}

interface FieldBodyProps {
  spec: ItemField;
  text: string;
  open: (id: string) => void;
}

interface FieldGroupProps {
  group: FieldGroupModel;
  shared: Omit<FieldProps, 'name'>;
}

interface CauseRowProps {
  kind: string;
  id: string;
  label: string;
  open: (id: string) => void;
}

interface FieldSelectProps {
  label: string;
  options: readonly Option[];
  value: string;
  onPick: (value: string) => void;
}

interface RefTagsProps {
  ids: readonly string[];
  open: (id: string) => void;
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
  AddChildProps, BlockedSectionProps, CaseRowProps, CauseRowProps, ChildRowProps, FieldBodyProps, FieldGroupModel, FieldGroupProps, FieldProps, FieldSelectProps,
  ItemPaneProps, NodeOpens, PaneBadge, PaneBodyProps, PaneCrumbsProps, PaneData, PaneFactsProps, PaneTab, PaneTabsProps, RefTagsProps, StageFact,
  TrailEdge, TrailLabelProps, TrailNode, TrailRowProps, TrailSectionProps, TrailTree,
};
