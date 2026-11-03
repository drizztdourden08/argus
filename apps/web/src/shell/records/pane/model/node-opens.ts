/* @layer renderer-app @kind logic */
import { ENTRY_LEAVES, LEAF_KINDS } from './pane-model.constants';
import type { NodeOpens, TrailNode } from '../trail/trail.type';

const across = (node: TrailNode): NodeOpens => ({ kind: 'board', href: `/p/${encodeURIComponent(node.project)}#${node.item}` });

const nodeOpens = (node: TrailNode, boardKey: string): NodeOpens | null => {
  if (!node.edge) return null;
  const here = node.project === boardKey;
  const type = node.type ?? '';
  if (ENTRY_LEAVES.includes(type)) return here ? { kind: 'entry', id: node.item } : across(node);
  if (type === 'test') return here ? { kind: 'test', id: node.item } : across(node);
  if (LEAF_KINDS.includes(type)) return null;
  return here ? { kind: 'item', id: node.item } : across(node);
};

export { nodeOpens };
