/* @layer renderer-app @kind types */
import type { PaneData } from '../pane.type';

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

export type { NodeOpens, TrailEdge, TrailLabelProps, TrailNode, TrailRowProps, TrailSectionProps, TrailTree };
