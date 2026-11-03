/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { RecordRow } from '../../../../../shell';
import { usePlanHead } from '../behavior/usePlanHead';
import type { PlanHeaderProps } from '../tests-props.type';

const PlanHeader = (props: PlanHeaderProps) => {
  const { plan } = props;
  const { code, open, head } = usePlanHead(plan);
  return (
    <Box onClick={(event) => event.preventDefault()}>
      <RecordRow head={head(`More actions for ${code} (group)`)} title={plan.item.title} onOpen={open} />
    </Box>
  );
};

export { PlanHeader };
