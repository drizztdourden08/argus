/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../../data/kanban';
import { itemCode } from '../../../../../../shell';
import { assigneeOf, stageLabel, typeOf } from '../../model';
import type { NodeContext, TreeNode } from '../tree.type';
import { caseNode } from './case-node';
import { casesOf } from './cases-of';
import { countOf } from './count-of';

const caseNodes = (plan: ItemRef, cx: NodeContext): TreeNode[] =>
  casesOf(plan).flatMap((id) => {
    const test = cx.tests.get(id);
    return test ? [caseNode(plan, id, test)] : [];
  });

const itemNode = (item: ItemRef, cx: NodeContext): TreeNode => {
  const children = [...(cx.byParent.get(item.id) ?? []).map((child) => itemNode(child, cx)), ...caseNodes(item, cx)];
  return {
    key: item.id,
    kind: 'item',
    id: item.id,
    typeId: typeOf(item),
    title: item.title,
    stage: stageLabel(item, 'assignee') ?? item.status,
    status: item.status,
    assignee: assigneeOf(item),
    open: itemCode(item),
    count: countOf(children),
    children,
  };
};

export { itemNode };
