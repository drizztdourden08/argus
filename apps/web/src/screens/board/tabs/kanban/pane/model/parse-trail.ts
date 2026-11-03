/* @layer renderer-app @kind logic */
import type { TrailEdge, TrailNode, TrailTree } from '../pane.type';
import type { Row } from './parse-trail.type';

const isRow = (value: unknown): value is Row => typeof value === 'object' && value !== null && !Array.isArray(value);

const text = (value: unknown): string | null => (typeof value === 'string' && value !== '' ? value : null);

const edgeOf = (value: unknown): TrailEdge | null => {
  if (!isRow(value) || typeof value.id !== 'string') return null;
  return { kind: text(value.kind) ?? 'item', id: value.id, state: text(value.state), since: text(value.since) };
};

const nodeOf = (value: unknown): TrailNode | null => {
  if (!isRow(value) || typeof value.item !== 'string') return null;
  const children = Array.isArray(value.children) ? value.children.map(nodeOf).filter((child): child is TrailNode => child !== null) : [];
  return {
    project: text(value.project) ?? '',
    item: value.item,
    title: text(value.title) ?? value.item,
    type: text(value.type),
    lane: text(value.lane),
    stage: text(value.stage),
    assignee: text(value.assignee),
    edge: edgeOf(value.edge),
    cycle: value.cycle === true,
    children,
  };
};

const parseTrail = (raw: unknown): TrailTree | null => {
  if (!isRow(raw) || typeof raw.depth !== 'number') return null;
  const root = nodeOf(raw.root);
  return root ? { depth: raw.depth, open: typeof raw.open === 'number' ? raw.open : 0, root } : null;
};

export { parseTrail };
