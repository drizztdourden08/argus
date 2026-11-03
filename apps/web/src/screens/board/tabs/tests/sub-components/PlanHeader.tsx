/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { RecordRow, useRecordPane } from '../../../../../shell';
import { planMenuGroups } from '../behavior/plan-menu-groups';
import { useTestsShared } from '../behavior/useTestsShared';
import { PLAN_KIND, planAssignee, planCode } from '../model';
import type { PlanHeaderProps } from '../tests-props.type';

const PlanHeader = (props: PlanHeaderProps) => {
  const { plan } = props;
  const { boardKey, threads } = useTestsShared();
  const pane = useRecordPane();
  const code = planCode(plan.item.id);
  const open = () => pane.open(code);
  return (
    <Box onClick={(event) => event.preventDefault()}>
      <RecordRow
        head={{
          kind: PLAN_KIND,
          code,
          assignee: planAssignee(plan.item, boardKey),
          boardKey,
          threads,
          menu: planMenuGroups(code, open),
          menuLabel: `More actions for ${code} (group)`,
        }}
        title={plan.item.title}
        onOpen={open}
      />
    </Box>
  );
};

export { PlanHeader };
