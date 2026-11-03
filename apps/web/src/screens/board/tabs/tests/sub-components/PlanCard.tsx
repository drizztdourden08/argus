/* @layer renderer-app @kind component */
import { Card } from '@drizztdourden08/tessera';
import { RecordCard, useRecordPane } from '../../../../../shell';
import { planMenuGroups } from '../behavior/plan-menu-groups';
import { useTestsShared } from '../behavior/useTestsShared';
import { PLAN_KIND, planAssignee, planCode, planTone } from '../model';
import type { PlanCardProps } from '../tests-props.type';
import { PlanFoot } from './PlanFoot';

const PlanCard = (props: PlanCardProps) => {
  const { plan } = props;
  const { boardKey, threads } = useTestsShared();
  const pane = useRecordPane();
  const code = planCode(plan.item.id);
  const open = () => pane.open(code);
  return (
    <Card edgeTone={planTone(plan.counts)} data-plan={code}>
      <RecordCard
        head={{
          kind: PLAN_KIND,
          code,
          assignee: planAssignee(plan.item, boardKey),
          boardKey,
          threads,
          menu: planMenuGroups(code, open),
          menuLabel: `More actions for ${code}`,
        }}
        title={plan.item.title}
        openLabel={`Open ${code}: ${plan.item.title}`}
        onOpen={open}
        foot={<PlanFoot plan={plan} code={code} />}
      />
    </Card>
  );
};

export { PlanCard };
