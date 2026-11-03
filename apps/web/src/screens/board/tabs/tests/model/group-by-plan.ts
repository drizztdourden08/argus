/* @layer renderer-app @kind logic */
import { groupRecords } from '../../../../../shell';
import type { RecordGroups } from '../../../../../shell';
import type { PlanView, TestDoc } from './model.type';

const groupByPlan = (tests: readonly TestDoc[], plans: readonly PlanView[]): RecordGroups<TestDoc> => {
  const keysOf = (test: TestDoc): string[] => plans.filter((plan) => plan.cases.includes(test.id)).map((plan) => plan.item.id);
  const info = (key: string) => {
    const at = plans.findIndex((plan) => plan.item.id === key);
    return at < 0 ? null : { rank: at, closed: plans[at]?.closed ?? false };
  };
  return groupRecords(tests, keysOf, info);
};

export { groupByPlan };
