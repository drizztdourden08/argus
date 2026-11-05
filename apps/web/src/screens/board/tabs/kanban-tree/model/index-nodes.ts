/* @layer renderer-app @kind logic */
import type { TreeNode } from '../tree.type';

const indexNodes = (nodes: readonly TreeNode[]): Map<string, TreeNode> =>
  new Map(nodes.flatMap((node): [string, TreeNode][] => [[node.key, node], ...indexNodes(node.children)]));

export { indexNodes };
