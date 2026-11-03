/* @layer renderer-app @kind logic */
import type { TreeNode } from '../tree.type';

const pruneTree = (nodes: readonly TreeNode[], keep: ReadonlySet<string>): TreeNode[] =>
  nodes.flatMap((node) => {
    const items = pruneTree(node.children.filter((child) => child.kind === 'item'), keep);
    const own = keep.has(node.key);
    if (!own && items.length === 0) return [];
    const cases = own ? node.children.filter((child) => child.kind === 'case') : [];
    return [{ ...node, children: [...items, ...cases] }];
  });

export { pruneTree };
