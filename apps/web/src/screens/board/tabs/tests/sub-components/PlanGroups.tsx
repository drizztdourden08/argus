/* @layer renderer-app @kind component */
import { Flex, SectionHeader, Text } from '@drizztdourden08/tessera';
import { RecordGroup } from '../../../../../shell';
import type { PlanView } from '../model';
import { CLOSED_PLANS, NO_PLAN_GROUP } from '../tests.constants';
import type { PlanGroupsProps } from '../tests-props.type';
import { CaseList } from './CaseList';
import { PlanGroup } from './PlanGroup';

const planOf = (plans: readonly PlanView[], key: string): PlanView | undefined => plans.find((plan) => plan.item.id === key);

const PlanGroups = (props: PlanGroupsProps) => {
  const { groups, plans } = props;
  const { none, closed } = groups;
  const total = groups.groups.length + closed.length + (none.length > 0 ? 1 : 0);
  return (
    <Flex direction="column" gap="sm" role="group" aria-label="Cases">
      <SectionHeader title="Cases" level={3} />
      {total === 0 && <Text tone="muted">No case in a plan yet.</Text>}
      {groups.groups.map((group) => {
        const plan = planOf(plans, group.key);
        return plan ? <PlanGroup key={group.key} plan={plan} tests={group.rows} /> : null;
      })}
      {none.length > 0 && (
        <RecordGroup header={<Text weight={500}>{NO_PLAN_GROUP}</Text>} count={none.length} label={NO_PLAN_GROUP}>
          <CaseList tests={none} />
        </RecordGroup>
      )}
      {closed.length > 0 && (
        <RecordGroup header={<Text weight={500}>{CLOSED_PLANS}</Text>} count={closed.length} label={CLOSED_PLANS} defaultOpen={false}>
          {closed.map((group) => {
            const plan = planOf(plans, group.key);
            return plan ? <PlanGroup key={group.key} plan={plan} tests={group.rows} /> : null;
          })}
        </RecordGroup>
      )}
    </Flex>
  );
};

export { PlanGroups };
