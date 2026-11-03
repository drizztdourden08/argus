/* @layer renderer-app @kind component */
import { RecordGroup } from '../../../../../shell';
import { planCode } from '../model';
import type { PlanGroupProps } from '../tests-props.type';
import { CaseList } from './CaseList';
import { PlanHeader } from './PlanHeader';

const PlanGroup = (props: PlanGroupProps) => {
  const { plan, tests } = props;
  return (
    <RecordGroup header={<PlanHeader plan={plan} />} count={tests.length} label={`Cases of ${planCode(plan.item.id)}`} defaultOpen={!plan.closed}>
      <CaseList tests={tests} />
    </RecordGroup>
  );
};

export { PlanGroup };
