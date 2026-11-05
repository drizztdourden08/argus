/* @layer renderer-app @kind component */
import { useTestsShared } from '../behavior/useTestsShared';
import { doneTests, isToRun, sortToRun } from '../model';
import type { CaseListProps } from '../tests-props.type';
import { CaseItems } from './CaseItems';

const CaseList = (props: CaseListProps) => {
  const { tests } = props;
  const { order, direction, showPassed } = useTestsShared();
  return <CaseItems tests={[...sortToRun(tests.filter(isToRun), order, direction), ...doneTests(tests, showPassed)]} />;
};

export { CaseList };
