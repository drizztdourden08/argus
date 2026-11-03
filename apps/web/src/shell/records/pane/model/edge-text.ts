/* @layer renderer-app @kind logic */
import { formatAgo } from '../../../../format';
import { EDGE_WORDS } from './pane-model.constants';
import type { TrailEdge } from '../pane.type';

const edgeText = (edge: TrailEdge, now: number): string => {
  const parts = [EDGE_WORDS[edge.kind] ?? edge.kind, edge.id, edge.state, edge.since ? `${formatAgo(edge.since, now)} ago` : null];
  return parts.filter(Boolean).join(' · ');
};

export { edgeText };
