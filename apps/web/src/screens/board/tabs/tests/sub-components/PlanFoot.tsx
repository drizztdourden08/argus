/* @layer renderer-app @kind component */
import { Button, Flex, Icon, Spacer, Tag, Text } from '@drizztdourden08/tessera';
import { Age, useRecordPane, useRunOps } from '../../../../../shell';
import { useTestsShared } from '../behavior/useTestsShared';
import { countsText } from '../model';
import { NO_CASES_TIP } from '../tests.constants';
import type { PlanFootProps } from '../tests-props.type';
import { BundleProgress } from './BundleProgress';

const bundleOf = (fields: Record<string, unknown> | undefined): string => {
  const bundle = fields?.bundle;
  return typeof bundle === 'string' ? bundle : '';
};

const PlanFoot = (props: PlanFootProps) => {
  const { plan, code } = props;
  const { boardKey } = useTestsShared();
  const pane = useRecordPane();
  const ops = useRunOps(boardKey, plan.item.id);
  const bundle = bundleOf(plan.item.fields);
  const empty = plan.cases.length === 0;
  const run = () => {
    if (plan.live) pane.open(code, 'run');
    else void ops.start().then((started) => started && pane.open(code, 'run'));
  };
  return (
    <Flex direction="column" gap="xs">
      <BundleProgress counts={plan.counts} />
      <Flex gap="sm" align="center">
        <Button size="sm" icon={<Icon name="play" />} disabled={empty} title={empty ? NO_CASES_TIP : undefined} onClick={run}>
          {plan.live ? 'Continue run' : 'Start run'}
        </Button>
        {bundle && <Tag title={`Bundle ${bundle}`}>{bundle}</Tag>}
        <Text variant="caption" tone="muted">{empty ? 'no cases' : countsText(plan.counts)}</Text>
        <Spacer />
        {plan.item.updatedAt && <Text variant="caption" tone="muted"><Age value={plan.item.updatedAt} /></Text>}
      </Flex>
    </Flex>
  );
};

export { PlanFoot };
