/* @layer renderer-app @kind logic */
import type { TrailNode } from '../pane.type';

const nodeRef = (node: TrailNode, boardKey: string): string =>
  node.project === boardKey || node.project === '' ? node.item : `${node.project}/${node.item}`;

export { nodeRef };
