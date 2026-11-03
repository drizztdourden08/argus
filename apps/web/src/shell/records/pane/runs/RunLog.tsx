/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { useCollection } from '../../../../data/react';
import { isLiveRun } from '../../../../data/runs';
import type { RunDoc } from '../../../../data/types';
import { RunLogBody, RunStopButton } from '../../../../screens/board/run-terminal';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import type { RecordBodyProps } from '../pane.type';
import { LOADING_TEXT } from './runs.constants';

const RunLog = (props: RecordBodyProps) => {
  const { boardKey, target } = props.data;
  const runs = useCollection<RunDoc>(boardKey, 'runs');
  const run = runs.docs.find((candidate) => candidate.id === target.id);
  if (!runs.ready) return <Text tone="muted">{LOADING_TEXT}</Text>;
  if (!run) return <EmptyLine>{`${target.code} is not on the board.`}</EmptyLine>;
  return (
    <Flex direction="column" gap="md">
      {isLiveRun(run) && <Flex justify="end"><RunStopButton run={run} boardKey={boardKey} /></Flex>}
      <RunLogBody run={run} boardKey={boardKey} />
    </Flex>
  );
};

export { RunLog };
