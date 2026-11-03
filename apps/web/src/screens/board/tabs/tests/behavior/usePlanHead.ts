/* @layer renderer-app @kind logic */
import { useRecordPane } from '../../../../../shell';
import type { RecordHeadProps } from '../../../../../shell/records';
import { PLAN_KIND, planAssignee, planCode } from '../model';
import type { PlanView } from '../model';
import { planMenuGroups } from './plan-menu-groups';
import { useTestsShared } from './useTestsShared';

interface PlanHead {
  code: string;
  open: () => void;
  head: (menuLabel: string) => RecordHeadProps;
}

const usePlanHead = (plan: PlanView): PlanHead => {
  const { boardKey, threads } = useTestsShared();
  const pane = useRecordPane();
  const code = planCode(plan.item.id);
  const open = () => pane.open(code);
  const head = (menuLabel: string): RecordHeadProps => ({
    kind: PLAN_KIND,
    code,
    assignee: planAssignee(plan.item, boardKey),
    boardKey,
    threads,
    menu: planMenuGroups(code, open),
    menuLabel,
  });
  return { code, open, head };
};

export { usePlanHead };
