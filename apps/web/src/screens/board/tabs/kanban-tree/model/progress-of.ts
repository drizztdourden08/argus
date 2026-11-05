/* @layer renderer-app @kind logic */
import type { TreeNode, TreeProgress } from '../tree.type';

const progressOf = (node: TreeNode): TreeProgress | null => {
  const { done, total } = node.count;
  if (total === 0) return null;
  const word = node.children.some((child) => child.kind === 'case') ? 'cases passed' : 'children done';
  return { text: `${done}/${total}`, title: `${done} of ${total} ${word}` };
};

export { progressOf };
