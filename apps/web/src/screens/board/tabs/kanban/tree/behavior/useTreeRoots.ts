/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { buildTree, pruneTree } from '../model';
import type { TreeNode, TreeSource } from '../tree.type';

const useTreeRoots = (source: TreeSource, shown: readonly string[]): TreeNode[] => {
  const { items, tests, archived } = source;
  const built = useMemo(() => buildTree({ items, tests, archived }), [items, tests, archived]);
  return useMemo(() => pruneTree(built, new Set(shown)), [built, shown]);
};

export { useTreeRoots };
