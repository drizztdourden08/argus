/* @layer renderer-app @kind component */
import { Disclosure, Flex, Grid, SectionHeader, Text } from '@drizztdourden08/tessera';
import { CLOSED_PLANS, NO_PLANS_TEXT } from '../tests.constants';
import type { PlanCardsProps } from '../tests-props.type';
import { PlanCard } from './PlanCard';

const PlanCards = (props: PlanCardsProps) => {
  const { plans } = props;
  const open = plans.filter((plan) => !plan.closed);
  const closed = plans.filter((plan) => plan.closed);
  return (
    <Flex direction="column" gap="sm" role="group" aria-label="Plans">
      <SectionHeader title="Plans" count={plans.length} level={3} />
      {plans.length === 0 && <Text tone="muted">{NO_PLANS_TEXT}</Text>}
      <Grid minColWidth={288} gap="md">
        {open.map((plan) => <PlanCard key={plan.item.id} plan={plan} />)}
      </Grid>
      {closed.length > 0 && (
        <Disclosure summary={`${CLOSED_PLANS} · ${closed.length}`} defaultOpen={false}>
          <Grid minColWidth={288} gap="md">
            {closed.map((plan) => <PlanCard key={plan.item.id} plan={plan} />)}
          </Grid>
        </Disclosure>
      )}
    </Flex>
  );
};

export { PlanCards };
