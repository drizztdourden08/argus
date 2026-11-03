/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { StepList } from '../../../../screens/board/tabs/changes';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import { useRecordPane } from '../../behavior/useRecordPane';
import { LOG_TAB, RUN_MISSING } from '../checks/checks.constants';
import { useCheckRun } from '../checks/useCheckRun';
import type { RecordBodyProps } from '../pane.type';

const StepsTab = (props: RecordBodyProps) => {
  const { boardKey, target } = props.data;
  const pane = useRecordPane();
  const { run, ready } = useCheckRun(boardKey, target.id);
  if (!ready) return <Text tone="muted">Reading the check run.</Text>;
  if (!run) return <EmptyLine>{RUN_MISSING}</EmptyLine>;
  return <StepList run={run} onLog={() => pane.setTab(LOG_TAB)} />;
};

export { StepsTab };
