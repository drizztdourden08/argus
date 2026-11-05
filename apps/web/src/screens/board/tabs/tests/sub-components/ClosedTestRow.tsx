/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { RecordRow } from '../../../../../shell';
import { useTestsShared } from '../behavior/useTestsShared';
import { caseAssignee } from '../model';
import { CASE_KIND } from '../tests.constants';
import type { DoneSummaryProps } from '../tests-props.type';
import { DoneSummary } from './DoneSummary';

const ClosedTestRow = (props: DoneSummaryProps) => {
  const { test } = props;
  const { boardKey, threads, onOpen } = useTestsShared();
  return (
    <Box data-swap-item data-test={test.id}>
      <RecordRow
        head={{ kind: CASE_KIND, code: test.id, assignee: caseAssignee(test), boardKey, threads, menu: [], menuLabel: `More actions for ${test.id}` }}
        title={test.title}
        onOpen={() => onOpen(test.id)}
        trailing={<DoneSummary test={test} />}
      />
    </Box>
  );
};

export { ClosedTestRow };
