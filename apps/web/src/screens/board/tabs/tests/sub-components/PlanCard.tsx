/* @layer renderer-app @kind component */
import { RecordCard } from '../../../../../shell';
import { usePlanHead } from '../behavior/usePlanHead';
import { planTone } from '../model';
import type { PlanCardProps } from '../tests-props.type';
import { PlanFoot } from './PlanFoot';

const PlanCard = (props: PlanCardProps) => {
  const { plan } = props;
  const { code, open, head } = usePlanHead(plan);
  return (
    <RecordCard
      head={head(`More actions for ${code}`)}
      title={plan.item.title}
      openLabel={`Open ${code}: ${plan.item.title}`}
      onOpen={open}
      edgeTone={planTone(plan.counts)}
      attrs={{ 'data-swap-item': '', 'data-plan': code }}
      foot={<PlanFoot plan={plan} code={code} />}
    />
  );
};

export { PlanCard };
