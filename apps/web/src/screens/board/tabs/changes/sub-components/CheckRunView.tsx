/* @layer renderer-app @kind component */
import { Disclosure, Flex, Status, Text } from '@drizztdourden08/tessera';
import { shortCommit } from '../../../../../format';
import { Age } from '../../../../../shell';
import { CHECK_STATUSES } from '../changes.constants';
import type { CheckRunViewProps } from '../changes.type';
import { StepList } from './StepList';

const CheckRunView = (props: CheckRunViewProps) => {
  const { run, first, onLog } = props;
  const summary = (
    <Flex gap="sm" align="center" wrap>
      <Status map={CHECK_STATUSES} value={run.status} dot size="sm" />
      <Text variant="caption" tone="muted" mono>{run.id}</Text>
      <Text variant="caption" tone="muted">{`${run.trigger} on ${shortCommit(run.commit)}`}</Text>
      {run.startedAt && <Text variant="caption" tone="muted"><Age value={run.startedAt} ago /></Text>}
    </Flex>
  );
  return (
    <Disclosure defaultOpen={first} summary={summary} size="sm">
      <Flex direction="column" gap="sm">
        {run.summary && <Text variant="caption" tone="muted">{run.summary}</Text>}
        <StepList run={run} onLog={onLog} />
      </Flex>
    </Disclosure>
  );
};

export { CheckRunView };
