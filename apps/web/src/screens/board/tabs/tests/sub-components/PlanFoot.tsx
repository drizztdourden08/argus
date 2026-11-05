/* @layer renderer-app @kind component */
import { Button, Flex, Icon, Spacer, Text } from '@drizztdourden08/tessera';
import { Age, useRecordPane, useRunOps } from '../../../../../shell';
import { useTestsShared } from '../behavior/useTestsShared';
import { countsText } from '../model';
import { NO_CASES_TIP } from '../tests.constants';
import type { PlanFootProps } from '../tests-props.type';
import { RunProgress } from './RunProgress';

const PlanFoot = (props: PlanFootProps) => {
  const { plan, code } = props;
  const { boardKey } = useTestsShared();
  const pane = useRecordPane();
  const ops = useRunOps(boardKey, plan.item.id);
  const empty = plan.cases.length === 0;
  const run = () => {
    if (plan.live) pane.open(code, 'runs');
    else void ops.start().then((started) => started && pane.open(code, 'runs'));
  };
  return (
    <Flex direction="column" gap="xs">
      <RunProgress counts={plan.counts} fill />
      <Flex gap="sm" align="center">
        <Button size="sm" icon={<Icon name="play" />} disabled={empty} title={empty ? NO_CASES_TIP : undefined} onClick={run}>
          {plan.live ? 'Continue run' : 'Start run'}
        </Button>
        <Text variant="caption" tone="muted">{empty ? 'no cases' : countsText(plan.counts)}</Text>
        <Spacer />
        {plan.item.updatedAt && <Text variant="caption" tone="muted"><Age value={plan.item.updatedAt} /></Text>}
      </Flex>
    </Flex>
  );
};

export { PlanFoot };
