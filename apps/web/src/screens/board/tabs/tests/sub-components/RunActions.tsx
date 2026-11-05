/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { AssigneeMark } from '../../../../../shell';
import { useTestsShared } from '../behavior/useTestsShared';
import type { RunActionsProps } from '../tests-head-props.type';
import { HandButton } from './HandButton';
import { RunCounts } from './RunCounts';
import { RunPlayButton } from './RunPlayButton';
import { RunProgress } from './RunProgress';

const RunActions = (props: RunActionsProps) => {
  const { view, runs } = props;
  const { boardKey, threads } = useTestsShared();
  const { item } = view;
  return (
    <Flex gap="xs" align="center" className="run-card__end">
      <RunCounts counts={view.counts} />
      <RunProgress counts={view.counts} />
      <RunPlayButton view={view} runs={runs} />
      {(view.stage === 'ready' || view.handed) && <HandButton run={item} handed={view.handed} />}
      <AssigneeMark assignee={item.assignee ?? item.thread ?? ''} boardKey={boardKey} threads={threads} />
    </Flex>
  );
};

export { RunActions };
