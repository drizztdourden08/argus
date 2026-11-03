/* @layer renderer-app @kind component */
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import { Evidence } from '../../../../screens/board/tabs/tests/sub-components/Evidence';
import type { RecordBodyProps } from '../pane.type';
import { useTestCase } from '../testcase/useTestCase';

const TestCaseMediaTab = (props: RecordBodyProps) => {
  const { data } = props;
  const { test } = useTestCase(data);
  if (!test) return <EmptyLine>{`No image on ${data.code} yet.`}</EmptyLine>;
  return <Evidence test={test} boardKey={data.boardKey} />;
};

export { TestCaseMediaTab };
