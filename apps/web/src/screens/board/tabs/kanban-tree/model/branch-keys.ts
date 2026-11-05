/* @layer renderer-app @kind logic */
import type { TreeNode } from '../tree.type';

const branchKeys = (nodes: readonly TreeNode[]): string[] =>
  nodes.flatMap((node) => (node.children.length > 0 ? [node.key, ...branchKeys(node.children)] : []));

export { branchKeys };
