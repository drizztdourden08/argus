/* @layer renderer-app @kind component */
import { Flex, Icon, IconButton, Spacer, Text } from '@drizztdourden08/tessera';
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
  const label = plan.live ? 'Continue run' : 'Start run';
  const run = () => {
    if (plan.live) pane.open(code, 'runs');
    else void ops.start().then((started) => started && pane.open(code, 'runs'));
  };
  return (
    <Flex direction="column" gap="xs">
      <RunProgress counts={plan.counts} fill />
      <Flex gap="sm" align="center">
        <IconButton size="xs" tone="accent" label={label} tooltip={empty ? NO_CASES_TIP : label} disabled={empty} onClick={run}>
          <Icon name="play" />
        </IconButton>
        <Text variant="caption" tone="muted">{empty ? 'no cases' : countsText(plan.counts)}</Text>
        <Spacer />
        {plan.item.updatedAt && <Text variant="caption" tone="muted"><Age value={plan.item.updatedAt} /></Text>}
      </Flex>
    </Flex>
  );
};

export { PlanFoot };
