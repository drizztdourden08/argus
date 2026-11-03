/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { stepStatus } from '../../../../screens/board/tabs/changes';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import { RUN_MISSING } from './checks.constants';
import type { CheckRunViewProps } from './checks.type';
import { StepLog } from './StepLog';

const CheckLog = (props: CheckRunViewProps) => {
  const { data, run, ready } = props;
  if (!ready) return <Text tone="muted">Reading the check run.</Text>;
  if (!run) return <EmptyLine>{RUN_MISSING}</EmptyLine>;
  const failing = run.steps.findIndex((step) => stepStatus(step) === 'fail');
  const first = failing === -1 ? 0 : failing;
  return (
    <Flex direction="column" gap="sm" role="list" aria-label={`Step logs of ${run.id}`}>
      {run.steps.map((step, index) => <StepLog key={`${index}-${step.name}`} boardKey={data.boardKey} run={run} index={index} first={index === first} />)}
    </Flex>
  );
};

export { CheckLog };
