/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import type { Test } from '../../../../../data/types';
import { itemCode } from '../../../../../shell';
import { OWNER } from '../../kanban/model';
import { CASE_TYPE, MANUAL_MODE, NO_COUNT } from '../tree.constants';
import type { TreeNode } from '../tree.type';

const caseNode = (plan: ItemRef, id: string, test: Test): TreeNode => ({
  key: `${plan.id}/${id}`,
  kind: 'case',
  id,
  typeId: CASE_TYPE,
  title: test.title,
  stage: test.status,
  status: test.status,
  assignee: test.mode === MANUAL_MODE ? OWNER : test.thread ?? OWNER,
  open: itemCode(plan),
  count: NO_COUNT,
  children: [],
});

export { caseNode };
