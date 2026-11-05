/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import { doneTests, isToRun, sortToRun } from '../model';
import { KEY_HINT } from '../tests.constants';
import type { TestSectionsProps } from '../tests-props.type';
import { CaseItems } from './CaseItems';
import { DoneList } from './DoneList';
import { SectionHead } from './SectionHead';

const TestSections = (props: TestSectionsProps) => {
  const { tests, emptyText } = props;
  const { order, direction, showPassed } = useTestsShared();
  const toRun = sortToRun(tests.filter(isToRun), order, direction);
  const done = doneTests(tests, showPassed);
  const hint = toRun.length > 0 ? <Text variant="caption" tone="muted" className="test-section__hint">{KEY_HINT}</Text> : undefined;
  return (
    <>
      <Box role="group" aria-label="To run">
        <SectionHead icon="list-checks" title="To run" count={toRun.length} action={hint} />
        {toRun.length > 0
          ? <CaseItems tests={toRun} />
          : <Text tone="muted" className="test-section__empty">{tests.length > 0 ? 'Nothing left to run.' : emptyText}</Text>}
      </Box>
      {done.length > 0 && <DoneList tests={done} />}
    </>
  );
};

export { TestSections };
