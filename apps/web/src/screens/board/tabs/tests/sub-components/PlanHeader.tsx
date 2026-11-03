/* @layer renderer-app @kind component */
import { RecordGroupHeader } from '../../../../../shell';
import { usePlanHead } from '../behavior/usePlanHead';
import { useTestsShared } from '../behavior/useTestsShared';
import { PLAN_KIND } from '../model';
import type { PlanHeaderProps } from '../tests-props.type';

const PlanHeader = (props: PlanHeaderProps) => {
  const { plan } = props;
  const { boardKey } = useTestsShared();
  const { code, open } = usePlanHead(plan);
  return <RecordGroupHeader kind={PLAN_KIND} code={code} title={plan.item.title} boardKey={boardKey} onOpen={open} />;
};

export { PlanHeader };
