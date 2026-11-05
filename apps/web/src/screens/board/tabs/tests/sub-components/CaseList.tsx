/* @layer renderer-app @kind component */
import { useTestsShared } from '../behavior/useTestsShared';
import { doneTests, isToRun, sortToRun } from '../model';
import type { CaseListProps } from '../tests-props.type';
import { CaseRow } from './CaseRow';

const CaseList = (props: CaseListProps) => {
  const { tests } = props;
  const { order, direction, showPassed } = useTestsShared();
  const rows = [...sortToRun(tests.filter(isToRun), order, direction), ...doneTests(tests, showPassed)];
  return <>{rows.map((test) => <CaseRow key={test.id} test={test} />)}</>;
};

export { CaseList };
