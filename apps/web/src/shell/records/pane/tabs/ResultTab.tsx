/* @layer renderer-app @kind component */
import { useBoardName } from '../../../../data/boards';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import type { RecordBodyProps } from '../pane.type';
import { CaseResult } from '../testcase/CaseResult';
import { useTestCase } from '../testcase/useTestCase';

const ResultTab = (props: RecordBodyProps) => {
  const { data } = props;
  const { test } = useTestCase(data);
  const nameOf = useBoardName();
  if (!test) return <EmptyLine>{`${data.code} is not on the board ${nameOf(data.boardKey)}.`}</EmptyLine>;
  return <CaseResult test={test} boardKey={data.boardKey} />;
};

export { ResultTab };
