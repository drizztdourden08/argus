/* @layer renderer-app @kind logic */
import type { NodeContext, TreeNode, TreeSource } from '../tree.type';
import { byRank } from './by-rank';
import { groupByParent } from './group-by-parent';
import { isLinked } from './is-linked';
import { itemNode } from './item-node';

const buildTree = (source: TreeSource): TreeNode[] => {
  const live = source.items.filter((item) => !item.deletedAt && (source.archived || !item.archivedAt));
  const byId = new Map(live.map((item) => [item.id, item]));
  const linked = live.filter((item) => isLinked(item, byId));
  const context: NodeContext = {
    byParent: groupByParent(linked),
    tests: new Map(source.tests.filter((test) => !test.deletedAt).map((test) => [test.id, test])),
  };
  return byRank(live.filter((item) => !isLinked(item, byId))).map((root) => itemNode(root, context));
};

export { buildTree };
