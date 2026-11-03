/* @layer renderer-app @kind logic */
import { CASE_DONE, DROPPED, ITEM_DONE } from '../tree.constants';
import type { TreeCount, TreeNode } from '../tree.type';

const isDone = (node: TreeNode): boolean => node.status === (node.kind === 'case' ? CASE_DONE : ITEM_DONE);

const countOf = (children: readonly TreeNode[]): TreeCount => {
  const live = children.filter((child) => child.status !== DROPPED);
  return { done: live.filter(isDone).length, total: live.length };
};

export { countOf };
