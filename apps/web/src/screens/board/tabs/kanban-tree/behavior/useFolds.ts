/* @layer renderer-app @kind logic */
import { useCallback, useMemo, useState } from 'react';
import { branchKeys, closedAfter, openKeys } from '../model';
import type { FoldState, TreeNode } from '../tree.type';

const useFolds = (roots: readonly TreeNode[]): FoldState => {
  const branches = useMemo(() => branchKeys(roots), [roots]);
  const [closed, setClosed] = useState<ReadonlySet<string>>(new Set());
  const expanded = useMemo(() => openKeys(branches, closed), [branches, closed]);
  const onChange = useCallback((keys: readonly string[]) => setClosed(closedAfter(branches, keys)), [branches]);
  const foldAll = useCallback(() => setClosed(new Set(branches)), [branches]);
  const unfoldAll = useCallback(() => setClosed(new Set()), []);
  return { expanded, onChange, foldAll, unfoldAll };
};

export { useFolds };
