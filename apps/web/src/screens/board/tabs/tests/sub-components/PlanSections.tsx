/* @layer renderer-app @kind component */
import { EmptyState, Flex } from '@drizztdourden08/tessera';
import { NO_MATCH_TEXT, NO_TESTS_TEXT } from '../tests.constants';
import type { PlanSectionsProps } from '../tests-props.type';
import { PlanCards } from './PlanCards';
import { PlanGroups } from './PlanGroups';

const PlanSections = (props: PlanSectionsProps) => {
  const { view } = props;
  const { plans, byPlan, filtered } = view;
  const cases = byPlan.groups.length + byPlan.closed.length + byPlan.none.length;
  if (plans.length === 0 && cases === 0) return <EmptyState message={filtered ? NO_MATCH_TEXT : NO_TESTS_TEXT} />;
  return (
    <Flex direction="column" gap="lg">
      <PlanCards plans={plans} />
      <PlanGroups groups={byPlan} plans={plans} />
    </Flex>
  );
};

export { PlanSections };
