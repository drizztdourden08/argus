/* @layer renderer-app @kind component */
import { Box, SectionHeader, Text } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import { doneTests, isToRun, sortToRun } from '../model';
import { KEY_HINT } from '../tests.constants';
import type { TestSectionsProps } from '../tests-props.type';
import { DoneList } from './DoneList';
import { SectionBand } from './SectionBand';
import { CaseRow } from './CaseRow';

const TestSections = (props: TestSectionsProps) => {
  const { tests, emptyText } = props;
  const { order, direction, showPassed } = useTestsShared();
  const toRun = sortToRun(tests.filter(isToRun), order, direction);
  const done = doneTests(tests, showPassed);
  return (
    <>
      <Box role="group" aria-label="To run">
        <SectionBand>
          <SectionHeader title="To run" count={toRun.length} level={4} action={toRun.length > 0 ? <Text variant="caption" tone="muted" className="test-section__hint">{KEY_HINT}</Text> : undefined} />
        </SectionBand>
        {toRun.length > 0
          ? toRun.map((test) => <CaseRow key={test.id} test={test} />)
          : <Text tone="muted" className="test-section__empty">{tests.length > 0 ? 'Nothing left to run.' : emptyText}</Text>}
      </Box>
      {done.length > 0 && <DoneList tests={done} />}
    </>
  );
};

export { TestSections };
