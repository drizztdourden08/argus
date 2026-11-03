/* @layer renderer-app @kind logic */
import { assigneeOfRecord } from '../../../../../shell';
import { kindOf } from '../../../../../data/registry';
import { PLAN_KIND } from './model.constants';
import type { ItemDoc } from './model.type';

const planAssignee = (plan: ItemDoc, boardKey: string): string => {
  const kind = kindOf(PLAN_KIND);
  return kind ? assigneeOfRecord(kind, { ...plan }, boardKey) : '';
};

export { planAssignee };
