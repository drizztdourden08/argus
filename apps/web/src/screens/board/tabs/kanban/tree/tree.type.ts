/* @layer renderer-app @kind types */
import type { ItemRef, ThreadChoice } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import type { Test } from '../../../../../data/types';
import type { KanbanView } from '../model';

type KanbanLayout = 'columns' | 'tree';

type TreeNodeKind = 'item' | 'case';

interface TreeCount {
  done: number;
  total: number;
}

interface TreeNode {
  key: string;
  kind: TreeNodeKind;
  id: string;
  typeId: string;
  title: string;
  stage: string;
  status: string;
  assignee: string;
  open: string;
  count: TreeCount;
  children: TreeNode[];
}

interface TreeProgress {
  text: string;
  title: string;
}

interface NodeContext {
  byParent: ReadonlyMap<string, readonly ItemRef[]>;
  tests: ReadonlyMap<string, Test>;
}

interface TreeSource {
  items: readonly ItemRef[];
  tests: readonly WithId<Test>[];
  archived: boolean;
}

interface LayoutState {
  layout: KanbanLayout;
  setLayout: (layout: KanbanLayout) => void;
}

interface FoldState {
  expanded: readonly string[];
  onChange: (keys: readonly string[]) => void;
  foldAll: () => void;
  unfoldAll: () => void;
}

interface TreeViewProps {
  boardKey: string;
  view: KanbanView;
  source: TreeSource;
  threads: readonly WithId<ThreadChoice>[];
  onOpen: (id: string) => void;
}

interface TreeRowProps {
  node: TreeNode;
  boardKey: string;
  threads: readonly WithId<ThreadChoice>[];
}

interface TreeFoldBarProps {
  fold: FoldState;
}

export type { FoldState, KanbanLayout, LayoutState, NodeContext, TreeCount, TreeFoldBarProps, TreeNode, TreeNodeKind, TreeProgress, TreeRowProps, TreeSource, TreeViewProps };
